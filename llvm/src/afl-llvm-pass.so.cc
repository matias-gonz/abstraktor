/*
  Copyright 2015 Google LLC All rights reserved.

  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at:

    http://www.apache.org/licenses/LICENSE-2.0

  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
*/

/*
   american fuzzy lop - LLVM-mode instrumentation pass
   ---------------------------------------------------

   Written by Laszlo Szekeres <lszekeres@google.com> and
              Michal Zalewski <lcamtuf@google.com>

   LLVM integration design comes from Laszlo Szekeres. C bits copied-and-pasted
   from afl-as.c are Michal's fault.

   This library is plugged into LLVM when invoking clang through afl-clang-fast.
   It tells the compiler to add code roughly equivalent to the bits discussed
   in ../afl-as.h.
*/

#define AFL_LLVM_PASS

#include "../include/config.h"
#include "../include/debug.h"

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <fstream>
#include <sstream>
#include <string>
#include <unordered_map>
#include <set>
#include <sys/shm.h>
#include <cxxabi.h>
#include <iostream>

#include "llvm/ADT/Statistic.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/IR/LegacyPassManager.h"
#include "llvm/IR/Module.h"
#include "llvm/Support/Debug.h"
#include "llvm/IR/DebugInfoMetadata.h"
#include "llvm/Transforms/IPO/PassManagerBuilder.h"
#include "llvm/IR/TypeFinder.h"
#include "llvm/Support/FileSystem.h"
#include "./targets_types.h"

#include "../rustc-demangle/crates/capi/include/rustc_demangle.h"
#include <nlohmann/json.hpp>

using namespace llvm;

#define TARGETS_TYPE std::unordered_map<std::string, std::set<int>>
#define CONST_TARGETS_TYPE std::unordered_map<std::string, std::unordered_map<int, std::string>>
//#define FUNC_TARGETS_TYPE std::unordered_map<std::string, std::unordered_map<int, std::unordered_map<std::string, std::vector<std::vector<unsigned int>>>>>
//#define BLOCK_TARGETS_TYPE std::unordered_map<std::string, std::unordered_map<int, std::unordered_map<std::string, std::vector<unsigned int>>>>


namespace
{

  struct ValueInfo {
    Type* type;
    std::vector<unsigned int> indexes;
  };


  class AFLCoverage : public ModulePass
  {

  public:
    static char ID;
    AFLCoverage() : ModulePass(ID) {}

    bool runOnModule(Module &M) override;

    // Global variables
    GlobalVariable *AFLMapPtr;
    GlobalVariable *AFLPrevLoc;
    unsigned int inst_ratio;

    // Types
    Type *VoidTy;
    Type *VoidPtrTy;
    PointerType *Int8PtrTy;
    IntegerType *Int8Ty;
    IntegerType *Int16Ty;
    IntegerType *Int32Ty;
    IntegerType *Int64Ty;
    PointerType *Int64PtrTy;
    

    // Store mapping data from basicblock location to ID
    std::ofstream bbToID;

    u16 *get_ID_ptr();
    static void get_debug_loc(const Instruction *I, std::string &Filename, unsigned &Line);
    static void load_instr_targets(TARGETS_TYPE &bb_targets, TargetsTypes &func_targets, TargetsTypes &block_targets, CONST_TARGETS_TYPE &const_targets);

    // -1: not checking, 0: not targets, 1: target BBs, 2: target functions, 3: target blocks, 4: target consts
    static u8 is_target_loc(std::string codefile, unsigned line, TARGETS_TYPE &bb_targets, TargetsTypes &func_targets, TargetsTypes &block_targets, CONST_TARGETS_TYPE &const_targets);

    u8 check_code_language(std::string codefile);
    void printFuncLog(std::string filename, unsigned line, u16 evtID, std::string func_name);
    void printBBLog(std::string filename, unsigned line, u16 evtID);
    void printBlockLog(std::string filename, unsigned line, u16 evtID);
    void printConstLog(std::string filename, unsigned line, u16 evtID, std::string const_name);
    std::vector<std::string> getArgumentTypeDebug(std::vector<std::string> instrumented_parameters, iterator_range<Function::arg_iterator> iterator_arguments);
    std::vector<std::pair<llvm::Value*, ValueInfo>> getArgument(const std::vector<std::string> &instrumented_parameters, iterator_range<Function::arg_iterator> iterator_arguments, std::vector<std::vector<unsigned int>> &default_indices);
    std::vector<llvm::Value*> getValues(std::vector<std::string> &vec, iterator_range<Function::arg_iterator> args, std::vector<std::vector<unsigned int>> &vec_selected_fields, IRBuilder<> &IRB);
    void changeStructPointersToStructTypes(std::vector<std::pair<llvm::Value*, ValueInfo>> &valueTypeMap);
    void extractValuesFromArgumentMap(std::vector<std::pair<llvm::Value*, ValueInfo>> &argument_map, IRBuilder<> &IRB,std::vector<llvm::Value*> &out_values); 
    Value* buildValuesArrayForFunction(std::vector<llvm::Value*> &values, IRBuilder<>& IRB);
    static void processTargets(const std::string &codefile, TargetsTypes &targets, const nlohmann::json &targets_json);
    bool isPointerToPointer(llvm::Value *v);
    bool isPointerToStruct(llvm::Value* v);
    bool isClangUnionType(llvm::Type *T);
    bool isPointerToUnion(llvm::Type *T);
    bool isUnionValue(llvm::Type *T);
  };

}

std::error_code EC;
llvm::raw_fd_ostream file2("mipass.log", EC, llvm::sys::fs::OpenFlags::OF_Append);

void AFLCoverage::processTargets(const std::string &codefile, TargetsTypes &targets, const nlohmann::json &targets_json) {
  for (auto it = targets_json.begin(); it != targets_json.end(); ++it) {
      TargetsTypes::LineNum line_num = std::stoul(it.key());
      const auto &variables_list = it.value();

      for (auto &var_obj : variables_list["var_info"]) {
          TargetsTypes::VarName var_name = var_obj["var_name"].get<TargetsTypes::VarName>();
          if (var_obj["struct_index_groups"].empty()) {
            std::vector<TargetsTypes::StructIndex> empty_struct_index_groups;
            targets.addStructIndexGroups(codefile, line_num, var_name, empty_struct_index_groups);
            continue;
          }
          for (auto &struct_indexes_row_json : var_obj["struct_index_groups"]) {
              TargetsTypes::StructIndexGroup struct_indexes_row = struct_indexes_row_json.get<TargetsTypes::StructIndexGroup>();

              targets.addStructIndexGroups(codefile, line_num, var_name, struct_indexes_row);
          }
      }
      
      const auto &group_json = variables_list["group"];
      targets.addGroups(
          codefile,
          line_num,
          group_json["end_mark"].get<bool>(),
          group_json["id"].get<TargetsTypes::GroupID>()
      );
  }
}


Value* AFLCoverage::buildValuesArrayForFunction(std::vector<llvm::Value*> &values, IRBuilder<>& IRB) {
    LLVMContext& Ctx = IRB.getContext();
    Type* Int32Ty = Type::getInt32Ty(Ctx);
    Type *VoidPtrTy = IRB.getInt8PtrTy();

    Function *F = IRB.GetInsertBlock()->getParent();
    IRBuilder<> EntryBuilder(&F->getEntryBlock(), F->getEntryBlock().getFirstInsertionPt());

    // void** arr = alloca(void*, values.size)
    Value* arr = EntryBuilder.CreateAlloca(
        VoidPtrTy,
        ConstantInt::get(Int32Ty, values.size())
    );

    for (size_t i = 0; i < values.size(); ++i) {
        Value* val = values[i];

        // T* alloc = alloca(T)
        Value* alloc = EntryBuilder.CreateAlloca(val->getType());

        // *alloc = val
        IRB.CreateStore(val, alloc);

        // void* casted = (void*)alloc
        Value* casted = IRB.CreateBitCast(alloc, VoidPtrTy);

        // arr[i]
        Value* gep = IRB.CreateGEP(
            VoidPtrTy,
            arr,
            ConstantInt::get(Int32Ty, i)
        );

        // arr[i] = casted
        IRB.CreateStore(casted, gep);
    }

    return arr;
}


bool AFLCoverage::isPointerToPointer(llvm::Value* v) {
    if (auto *ptrTy = llvm::dyn_cast<llvm::PointerType>(v->getType())) {
        return ptrTy->getElementType()->isPointerTy();
    }
    return false;
}

bool AFLCoverage::isPointerToStruct(llvm::Value* v) {
    auto ptrTy = v->getType();
    bool isStruct = true;
    if (ptrTy->isPointerTy()) {
        auto *elem = ptrTy->getPointerElementType();
        isStruct = elem->getPointerElementType()->isStructTy();
    }
    return isStruct;
}

bool AFLCoverage::isClangUnionType(llvm::Type *T) {
    auto *ST = llvm::dyn_cast<llvm::StructType>(T);
    if (!ST) return false;

    llvm::StringRef name = ST->getName();
    return name.startswith("union.") || name.contains("union");
}


bool AFLCoverage::isUnionValue(llvm::Type *T) {
    return isClangUnionType(T);
}

bool AFLCoverage::isPointerToUnion(llvm::Type *T) {
    if (!T->isPointerTy()) return false;
    return isClangUnionType(T->getPointerElementType());
}

void AFLCoverage::extractValuesFromArgumentMap(
    std::vector<std::pair<llvm::Value*, ValueInfo>> &argument_map,
    IRBuilder<> &IRB,
    std::vector<llvm::Value*> &out_values
) {
    changeStructPointersToStructTypes(argument_map);
    for (auto &pair : argument_map) {
      std::vector<llvm::Value*> tmp;
      llvm::Value* target_value = pair.first;
      llvm::Type* target_type = pair.second.type;
      if (pair.second.indexes.empty()) {
          out_values.push_back(target_value);
          continue;
      }
      for(auto &selected_field: pair.second.indexes){
        llvm::Value* zero  = llvm::ConstantInt::get(IRB.getInt32Ty(), 0);
        llvm::Value* offset = llvm::ConstantInt::get(IRB.getInt32Ty(), selected_field);
        llvm::Value* target_ptr = IRB.CreateGEP(
              target_type,
              target_value,
              {zero, offset}
          );
        
        //llvm::Type* field_ty = target_ptr->getType();
       

        llvm::Value* field_value = IRB.CreateLoad(target_ptr);

        llvm::Type* field_ty = field_value->getType();


        //field_ty->print(file2);
        //file2 << "\n";

        // union by value
        if (isUnionValue(field_ty)) {

            std::string wanted = "struct.candidate_state";
            llvm::StructType *realStruct = IRB.GetInsertBlock()->getModule()->getTypeByName(wanted);

            llvm::Value *casted = IRB.CreateBitCast(target_ptr, realStruct->getPointerTo());

            target_value = casted;
            target_type  = realStruct;
            continue;
        }


        if (isPointerToUnion(field_ty)) {


          std::string wanted = "struct.candidate_state";

          llvm::StructType *realStruct = IRB.GetInsertBlock()->getModule()->getTypeByName(wanted);

          llvm::Value *casted =
              IRB.CreateBitCast(field_value, realStruct->getPointerTo());

          target_value = casted;
          target_type  = realStruct;
          continue;
        }

        // struct*
        if (field_ty->isPointerTy() &&
            field_ty->getPointerElementType()->isStructTy()) {

            target_value = field_value;
            target_type = field_ty->getPointerElementType();
            continue;
        }

        // struct inside struct
        if (field_ty->isStructTy()) {
            llvm::AllocaInst* tmp = IRB.CreateAlloca(field_ty);
            IRB.CreateStore(field_value, tmp);

            target_value = tmp;
            target_type = field_ty;
            continue;
        }

        // primitivo u otra cosa
        out_values.push_back(field_value);
        break;
        
      }
    }
  }


std::vector<llvm::Value*> AFLCoverage::getValues(
                                    std::vector<std::string> &vec, 
                                    iterator_range<Function::arg_iterator> args, 
                                    std::vector<std::vector<unsigned int>> &vec_selected_fields, IRBuilder<> &IRB
                                  ){
    std::vector<std::pair<llvm::Value*, ValueInfo>> argument_map = getArgument(vec, args, vec_selected_fields);
    std::vector<llvm::Value*> res;
    extractValuesFromArgumentMap(argument_map, IRB, res);
    return res;
}


/***
 * Load identified interesting basicblocks(targets) to instrument
 ***/
void AFLCoverage::load_instr_targets(TARGETS_TYPE &bb_targets, TargetsTypes &func_targets, TargetsTypes &block_targets, CONST_TARGETS_TYPE &const_targets)
{
  char *target_file = getenv("TARGETS_FILE");
  //file2 << "Target File: " << target_file  << "\n";
  if (!target_file) {
    outs() << "[!!] TARGETS_FILE environment variable not set\n";
    return;
  }

  std::ifstream file(target_file);
  if (!file.is_open()) {
    outs() << "[!!] Could not open " << target_file << "\n";
    return;
  }

  nlohmann::json json;
  file >> json;

  if (!json.is_array()) {
    outs() << "[!!] JSON must be an array\n";
    return;
  }

  for (const auto& target : json) {
    if (target.find("path") == target.end() || target.find("targets_block") == target.end()) {
      outs() << "[!!] Missing required JSON fields in target object\n";
      continue;
    }

    std::string codefile = target["path"];

    auto targets_func_json = target["targets_function"];

    if (targets_func_json.is_object()) processTargets(codefile, func_targets, targets_func_json);
    
    auto targets_block = target["targets_block"];
  
    if (targets_block.is_object()) processTargets(codefile, block_targets, targets_block);
    
    
    auto targets_const = target["targets_const"];
    if (targets_const.is_object()) {
      for (auto it = targets_const.begin(); it != targets_const.end(); ++it) {
        unsigned line_num = std::stoul(it.key());
        std::string const_name = it.value().get<std::string>();

        const_targets[codefile][line_num] = const_name;
      }
    }
  }
}

/***
 * Check if current location is target: 1 for BB, 2 for function, 3 for block, 4 for const, 0 for not targets, -1 for not checking
 ***/
u8 AFLCoverage::is_target_loc(std::string codefile, unsigned line, TARGETS_TYPE &bb_targets, TargetsTypes &func_targets, TargetsTypes &block_targets, CONST_TARGETS_TYPE &const_targets)
{


  if (bb_targets.count(codefile))
  {
    std::set<int> locs = bb_targets[codefile];
    for (auto ep = locs.begin(); ep != locs.end(); ep++)
    {
      if (*ep == line)
      {
        bb_targets[codefile].erase(line);
        return 1;
      }
    }
  }

  if (func_targets.containsLine(codefile, line)) {
    if (func_targets.isFinalMark(codefile, line)) return 2;
    return 5;
  }
  
  if (block_targets.containsLine(codefile, line)) {
    if (block_targets.isFinalMark(codefile, line)) return 3;
    return 6;
  }

  if (const_targets.count(codefile))
  {

    auto& const_map = const_targets[codefile];
    if (const_map.count(line))
    {
      return 4;
    }
  }
  
  return 0;
}

/***
 * Get filename and location given one instruction
 ***/
void AFLCoverage::get_debug_loc(const Instruction *I, std::string &Filename, unsigned &Line)
{
  if (DILocation *Loc = I->getDebugLoc())
  {
    Line = Loc->getLine();
    Filename = Loc->getFilename().str();

    char *path = realpath(Filename.c_str(), NULL);
    if (path)
    {
      Filename = std::string(path);
    }
    else
    {
      std::string dir = Loc->getDirectory().str();
      if (dir.size() > 0)
      {
        Filename = dir + "/" + Filename;
      }
    }

    if (Filename.empty())
    {
      DILocation *oDILoc = Loc->getInlinedAt();
      if (oDILoc)
      {
        Line = oDILoc->getLine();
        Filename = oDILoc->getFilename().str();
        char *path = realpath(Filename.c_str(), NULL);
        if (path)
        {
          Filename = std::string(path);
        }
        else
        {
          std::string dir = Loc->getDirectory().str();
          if (dir.size() > 0)
          {
            Filename = dir + "/" + Filename;
          }
        }
      }
    }
  }
}

/***
 * Assign event ID in increasing order, instead of random assignment in AFL
 ***/
u16 *AFLCoverage::get_ID_ptr()
{
  // Create the shared memory if it does not exist, otherwise get the existing one
  int shmid = shmget((key_t)SHM_ID_KEY, sizeof(u16), IPC_CREAT | IPC_EXCL | 0666);
  if (shmid != 0)
  {
    shmid = shmget((key_t)SHM_ID_KEY, sizeof(u16), 0666);
  }

  // FIXME: If compilation is done in Docker (in subsequent RUN layers), the
  // shared memory is not carried over between layers, so IDs will conflict

  u16 *_id_ptr;
  if (shmid >= 0)
  {
    _id_ptr = (u16 *)shmat(shmid, NULL, 0);
    if (_id_ptr == (u16 *)-1)
    {
      ABORT("!!! shared memory error: fail to connect");
      _exit(1);
    }
    return _id_ptr;
  }
  else
  {
    ABORT("!!! shared memory error: fail to create");
    _exit(1);
  }
}

u8 AFLCoverage::check_code_language(std::string codefile)
{
  // Check if the code is written in Rust (return 1) or C/C++ (return 2)
  if (codefile.find(".rs") != std::string::npos)
  {
    return 1;
  }
  else
  {
    return 2;
  }
}

/***
 * Print compilation log
 ***/
void AFLCoverage::printFuncLog(std::string filename, unsigned line, u16 evtID, std::string func_name)
{
  OKF("Instrument %u at %s: at line %u for function %s", evtID, filename.c_str(), line, func_name.c_str());
  bbToID << evtID << ": at " << filename << " ; at line " << line << " for function " << func_name << std::endl;
}

void AFLCoverage::printBBLog(std::string filename, unsigned line, u16 evtID)
{
  bbToID << evtID << ": at " << filename << " ; at line " << line << " for block" << std::endl;
  OKF("Instrument %u at %s: at line %u for block", evtID, filename.c_str(), line);
}

void AFLCoverage::printBlockLog(std::string filename, unsigned line, u16 evtID)
{
  bbToID << evtID << ": at " << filename << " ; at line " << line << " for block" << std::endl;
  OKF("Instrument %u at %s: at line %u for block", evtID, filename.c_str(), line);
}

void AFLCoverage::printConstLog(std::string filename, unsigned line, u16 evtID, std::string const_name)
{
  bbToID << evtID << ": at " << filename << " ; at line " << line << " for const " << const_name << std::endl;
  OKF("Instrument %u at %s: at line %u for const %s", evtID, filename.c_str(), line, const_name.c_str());
}

void AFLCoverage::changeStructPointersToStructTypes(std::vector<std::pair<llvm::Value*, ValueInfo>> &valueTypeMap) {
    for (auto &pair : valueTypeMap) {
        llvm::Type* type = pair.second.type;

        if (type->isPointerTy()) {
            llvm::Type* elemTy = type->getPointerElementType();

            if (auto *structTy = dyn_cast<StructType>(elemTy)) {
                pair.second.type = structTy; 
            }
        }
    }
}
std::vector<std::string> AFLCoverage::getArgumentTypeDebug(std::vector<std::string> instrumented_parameters, iterator_range<Function::arg_iterator> iterator_arguments){
  std::vector<std::string> typeStrs;
  for(auto instrumented_parameter: instrumented_parameters){
    for(auto &Arg : iterator_arguments){
      if(instrumented_parameter == Arg.getName().str()){
        std::string typeStr;
        llvm::raw_string_ostream rso(typeStr);
        Arg.getType()->print(rso);
        rso.flush();
        typeStrs.push_back(typeStr);
      }
    }
  }
  return typeStrs;
}

std::vector<std::pair<llvm::Value*, ValueInfo>> AFLCoverage::getArgument(
    const std::vector<std::string> &instrumented_parameters,
    iterator_range<Function::arg_iterator> iterator_arguments,
    std::vector<std::vector<unsigned int>> &default_indices) 
{
    std::vector<std::pair<llvm::Value*, ValueInfo>> valueContainer;
    int idx = 0;
    for (auto &Arg : iterator_arguments) {
        for (const std::string &param : instrumented_parameters) {
            if (param == Arg.getName().str()) {
                struct ValueInfo valueInfoTmp;
                valueInfoTmp.type = Arg.getType();
                valueInfoTmp.indexes = default_indices[idx];
                valueContainer.push_back(std::make_pair(&Arg, valueInfoTmp));
                idx++;
            }
        }
    }

    return valueContainer;
}


char AFLCoverage::ID = 0;

bool AFLCoverage::runOnModule(Module &M)
{
  LLVMContext &C = M.getContext();

  VoidTy = Type::getVoidTy(C);
  Int8PtrTy = Type::getInt8PtrTy(C);
  Int8Ty = IntegerType::getInt8Ty(C);
  Int16Ty = IntegerType::getInt16Ty(C);
  Int32Ty = IntegerType::getInt32Ty(C);
  Int64Ty = IntegerType::getInt64Ty(C);
  Int64PtrTy = Type::getInt64PtrTy(C);
  VoidPtrTy = Type::getInt8PtrTy(C);

  /* Show a banner */

  // char be_quiet = 0;

  if (isatty(2) && !getenv("AFL_QUIET"))
  {
    SAYF(cCYA "afl-llvm-pass " cBRI VERSION cRST " by <lszekeres@google.com>\n");
  }

  /* Decide the size of instrumented functions */
  char *instr_func_size_str = getenv("INST_FUNC_SIZE");

  // Set one large number to disable it if the environment variable is not set
  unsigned int instr_func_size = 65536;
  if (instr_func_size_str)
  {
    if (sscanf(instr_func_size_str, "%u", &instr_func_size) != 1)
      FATAL("Bad value of INST_FUNC_SIZE");
  }

  if (getenv("USE_TRADITIONAL_BRANCH")){
    /* Decide instrumentation ratio */
    char *inst_ratio_str = getenv("AFL_INST_RATIO");
    inst_ratio = 100;
    if (inst_ratio_str)
    {
      if (sscanf(inst_ratio_str, "%u", &inst_ratio) != 1 || !inst_ratio ||
          inst_ratio > 100)
        FATAL("Bad value of AFL_INST_RATIO (must be between 1 and 100)");
    }

    /* Get globals for the SHM region and the previous location. Note that
      __afl_prev_loc is thread-local. */
    AFLMapPtr =
        new GlobalVariable(M, PointerType::get(Int8Ty, 0), false,
                          GlobalValue::ExternalLinkage, 0, "__afl_area_ptr");

    AFLPrevLoc = new GlobalVariable(
        M, Int32Ty, false, GlobalValue::ExternalLinkage, 0, "__afl_prev_loc",
        0, GlobalVariable::GeneralDynamicTLSModel, 0, false);
  }

  TARGETS_TYPE bb_targets;
  CONST_TARGETS_TYPE const_targets;
  TargetsTypes func_targets;
  TargetsTypes block_targets;
  std::map<TargetsTypes::GroupID, std::vector<Value*>> groupsPointerValues;
  std::set<std::pair<std::string, int>> instrumented_const_targets;
  load_instr_targets(bb_targets, func_targets, block_targets, const_targets);
  u8 codeLang = 0;

  static const std::string Xlibs("/usr/");

  for (auto &F : M) {
  //   // Label if this function is instrumented
    bool isTargetFunc = false;

    std::string filename;
    unsigned line = 0;
    unsigned const_line = 0;
    unsigned targetLine = 0;
    
    bool notBreakFunction = false;
    for (auto &BB : F) 
    {
      std::set<std::tuple<unsigned, bool, llvm::Value*, llvm::Instruction*, llvm::BasicBlock*>> block_lines;
      BasicBlock::iterator IP = BB.getFirstInsertionPt();

      // in each basic block, check if it is a target
      bool isTargetBlockEvent = false;
      bool isTargetConstEvent = false;
      bool notBreak = false;
      llvm::Value* valueOperandLeftSize;
      llvm::Instruction* nextI;
      llvm::BasicBlock* nextIFinal;
      for (auto &I : BB)
      {
        get_debug_loc(&I, filename, line);

        if (filename.empty() || line == 0 || !filename.compare(0, Xlibs.size(), Xlibs))
        {
          continue;
        }
        u16 isTarget = is_target_loc(filename, line, bb_targets, func_targets, block_targets, const_targets);

        file2 << "Target: " << isTarget << " For: " << F.getName().str() << ":" << line << "\n";

        if (isTarget == 2 || isTarget == 5)
        {
          targetLine = line;
          isTargetFunc = true;
          if (isTarget == 5) notBreakFunction = true;
        }
        else if (isTarget == 3 || isTarget == 6)
        {

          isTargetBlockEvent = true;
          if (auto *SI = dyn_cast<StoreInst>(&I)) {
            valueOperandLeftSize = SI->getValueOperand();
            nextI = I.getNextNode();
            if (nextI == nullptr) nextIFinal = I.getParent();
            block_lines.insert(std::make_tuple(line, isTarget == 6, valueOperandLeftSize, nextI, nextIFinal));
          }
        }
        else if (isTarget == 4)
        {
          isTargetConstEvent = true;
          const_line = line;
        }
      }

      /* skip if no target found or instrumented, and also not selected */
      if (!isTargetBlockEvent && !isTargetConstEvent && AFL_R(100) >= inst_ratio)
      {
        continue;
      }

      if (isTargetFunc) {

        Instruction *InsertPoint = &(*F.getEntryBlock().getFirstInsertionPt());
        IRBuilder<> IRB(InsertPoint);
        isTargetFunc = false;
        
        std::vector<std::vector<unsigned int>> vec_selected_fields;

        std::vector<std::string> vec;
        if (!func_targets.containsLine(filename, targetLine)) {
          get_debug_loc(&(*InsertPoint), filename, targetLine);
          continue;
        }

        TargetsTypes::LineEntries entries;
        
        if (!func_targets.getLineEntries(filename, targetLine, entries)) {
            continue;
        }

        for (const auto& entry : entries) {
          const auto& var = entry.first;
          const auto& index_row = entry.second;

          vec.push_back(var);
          vec_selected_fields.push_back(index_row);
        }

        std::vector<llvm::Value*> res = getValues(vec, F.args(), vec_selected_fields, IRB);
      
        if(res.size() == 0){

        } else {

          TargetsTypes::GroupID groupID = func_targets.getGroupID(filename, targetLine);

          std::vector<llvm::Value*> v = groupsPointerValues[groupID]; 

          v.insert(v.end(), res.begin(), res.end());

          if (notBreakFunction) {
              file2 << "FUNC accumulate for " << F.getName().str() << " group=" << groupID << " size=" << v.size() << "\n";
              groupsPointerValues[groupID] = v;
          } else {

          
            Value* arr = buildValuesArrayForFunction(v, IRB);

            groupsPointerValues.erase(groupID);

            u16 *evtIDPtr = get_ID_ptr();
            u16 evtID = *evtIDPtr;

            file2 << "Instrumenting function event for " << F.getName().str() << " with evtID " << evtID << " values \n";
            Value *evtValue = ConstantInt::get(Int16Ty, evtID);

            // Cast to double pointer
            Value* arrPtr = IRB.CreateBitCast(arr, PointerType::getUnqual(VoidPtrTy));

            //Get double pointer type
            Type *VoidPtrPtrTy = PointerType::getUnqual(VoidPtrTy); 

            auto *helperTy_const = FunctionType::get(VoidTy, {Int16Ty, Int8PtrTy, VoidPtrPtrTy, Int64Ty}, false);
            auto helper_const = M.getOrInsertFunction("trigger_func_event", helperTy_const);

            std::string function_name = F.getName().str();
              

            Value* function_name_value = IRB.CreateGlobalString(StringRef(function_name),"varName");
            IRB.CreateCall(helper_const, {evtValue, function_name_value, arrPtr, ConstantInt::get(Int64Ty, v.size())});
            
            /* increase counter */
            *evtIDPtr = ++evtID;
            get_debug_loc(&(*InsertPoint), filename, targetLine);
            std::string func_name = F.getName().str();
            if (codeLang == 0)
            {
              codeLang = check_code_language(filename);
            }

            if (codeLang == 2)
            {
              int demangled_status = -1;
              char *demangled_char = abi::__cxa_demangle(F.getName().data(), nullptr,
                                                        nullptr, &demangled_status);
              if (demangled_status == 0)
              {
                func_name = demangled_char;
              }
            }
            else
            {
              int demangled_status = -1;
              char *demangled_char = rustc_demangle(F.getName().data(), &demangled_status);
              if (demangled_status == 0)
              {
                func_name = demangled_char;
              }
            }
            printFuncLog(filename, line, evtID, func_name);

            /* increase counter */
            //*evtIDPtr = ++evtID;
          }
        }
      }
      
      if (isTargetBlockEvent)
      {
        for (auto [block_line, not_break, valueOperandLeftSize, nextI, nextIFinal] : block_lines) {
          file2 << "Processing block event for line " << block_lines.size() << "\n";
          Instruction *Pos = nextI ? nextI : nextIFinal->getTerminator();
          IRBuilder<> IRB(Pos);

          std::vector<std::pair<llvm::Value*, ValueInfo>> argument_map;

          TargetsTypes::LineEntries entries;

          // Can we have a function that returns the index only for blocks?
          if (!block_targets.getLineEntries(filename, block_line, entries)) {
            continue;
          }

          llvm::Type* valueOperandLeftSizeType = valueOperandLeftSize->getType();

          for (const auto& entry : entries) {
            struct ValueInfo valueInfo;
            valueInfo.type = valueOperandLeftSizeType;
            valueInfo.indexes = entry.second;
            argument_map.push_back(std::make_pair(valueOperandLeftSize, valueInfo));
          }

          std::vector<llvm::Value*> res;
          extractValuesFromArgumentMap(argument_map, IRB, res);
          
          if(res.size() == 0){

          } else {
              
              u16 *evtIDPtr = get_ID_ptr();
              u16 evtID = *evtIDPtr;
              
              TargetsTypes::GroupID groupID = block_targets.getGroupID(filename, block_line);

              std::vector<llvm::Value*>v = groupsPointerValues[groupID];
              file2 << "V (from groupsPointerValues[" << groupID << "]): " << v.size() << " + res: " << res.size() << "\n";
  
              v.insert(v.end(), res.begin(), res.end());

              groupsPointerValues[groupID] = v;

              if (not_break) {
                // nada más que hacer
              } else {
              
                Value *evtValue = ConstantInt::get(Int16Ty, evtID);
                groupsPointerValues.erase(groupID);
                // Cast to double pointer
            
                Value* arr = buildValuesArrayForFunction(v, IRB);

                Value* arrPtr = IRB.CreateBitCast(arr, PointerType::getUnqual(VoidPtrTy));

                //Get double pointer type
                Type *VoidPtrPtrTy = PointerType::getUnqual(VoidPtrTy); 

                auto *helperTy_const = FunctionType::get(VoidTy, {Int16Ty, Int8PtrTy, VoidPtrPtrTy, Int64Ty}, false);
                //file2 << "Not breaking function: " << F.getName().str() << " with " << v.size() << " values \n";
                file2 << "Instrumenting block event for " << F.getName().str() << " with evtID " << evtID << " values \n";

                auto helper_const = M.getOrInsertFunction("trigger_block_event", helperTy_const);

                std::string function_name = F.getName().str();
                  
                Value* function_name_value = IRB.CreateGlobalString(StringRef(function_name),"varName");
                IRB.CreateCall(helper_const, {evtValue, function_name_value, arrPtr, ConstantInt::get(Int64Ty, v.size())});

                /* store BB ID info */
                printBlockLog(filename, block_line, evtID);

                /* increase counter */
                *evtIDPtr = ++evtID;
              }
            }
        }
      }
      
      // if (false)
      // {

        
      //   std::pair<std::string, int> const_key = std::make_pair(filename, const_line);
      //   if (instrumented_const_targets.find(const_key) == instrumented_const_targets.end())
      //   {
      //     instrumented_const_targets.insert(const_key);
          
      //     std::string constName = const_targets[filename][const_line];
      //     u16 *evtIDPtr = get_ID_ptr();
      //     u16 evtID = *evtIDPtr;
      //     Value *evtValue = ConstantInt::get(Int16Ty, evtID);

      //     // Create a global string constant for the const name
      //     Value *constNameValue = IRB.CreateGlobalString(StringRef(constName), "const_name");

      //     auto *helperTy_const = FunctionType::get(VoidTy, {Int16Ty, Int8PtrTy, Int8PtrTy}, false);
      //     auto helper_const = M.getOrInsertFunction("trigger_const_event", helperTy_const);
          
      //     std::string function_name = F.getName().str();
      //     Value* function_name_value = IRB.CreateGlobalString(StringRef(function_name),"varName");

      //     IRB.CreateCall(helper_const, {evtValue, function_name_value, constNameValue});

      //     /* store const ID info */
      //     printConstLog(filename, const_line, evtID, constName);

      //     /* increase counter */
      //     *evtIDPtr = ++evtID;
      //   }
      // }
    
      if (getenv("USE_TRADITIONAL_BRANCH")){
        Instruction *Term = BB.getTerminator();
        IRBuilder<> IRB(Term);
        // Instrument all basicblocks to compute AFL feedback
        unsigned int cur_loc = AFL_R(MAP_SIZE);

        ConstantInt *CurLoc = ConstantInt::get(Int32Ty, cur_loc);

        /* Load prev_loc */

        LoadInst *PrevLoc = IRB.CreateLoad(AFLPrevLoc);
        PrevLoc->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));
        Value *PrevLocCasted = IRB.CreateZExt(PrevLoc, IRB.getInt32Ty());

        /* Load SHM pointer */

        LoadInst *MapPtr = IRB.CreateLoad(AFLMapPtr);
        MapPtr->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));
        Value *MapPtrIdx =
            IRB.CreateGEP(MapPtr, IRB.CreateXor(PrevLocCasted, CurLoc));

        /* Update bitmap */

        LoadInst *Counter = IRB.CreateLoad(MapPtrIdx);
        Counter->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));
        Value *Incr = IRB.CreateAdd(Counter, ConstantInt::get(Int8Ty, 1));
        IRB.CreateStore(Incr, MapPtrIdx)
            ->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));

        /* Set prev_loc to cur_loc >> 1 */

        StoreInst *Store =
            IRB.CreateStore(ConstantInt::get(Int32Ty, cur_loc >> 1), AFLPrevLoc);
        Store->setMetadata(M.getMDKindID("nosanitize"), MDNode::get(C, None));
      }    
    }

    /* Instrument function if it is one target or the size is above threshold */
    // if (isTargetFunc || F.getInstructionCount() > instr_func_size)
    
  }
  
  file2.close();
  return true;
}

static void registerAFLPass(const PassManagerBuilder &,
                            legacy::PassManagerBase &PM)
{

  PM.add(new AFLCoverage());
}

static RegisterStandardPasses RegisterAFLPass(
    PassManagerBuilder::EP_ModuleOptimizerEarly, registerAFLPass);

static RegisterStandardPasses RegisterAFLPass0(
    PassManagerBuilder::EP_EnabledOnOptLevel0, registerAFLPass);
