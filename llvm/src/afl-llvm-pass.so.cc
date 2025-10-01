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

#include "../rustc-demangle/crates/capi/include/rustc_demangle.h"
#include <nlohmann/json.hpp>

using namespace llvm;

#define TARGETS_TYPE std::unordered_map<std::string, std::set<int>>
#define CONST_TARGETS_TYPE std::unordered_map<std::string, std::unordered_map<int, std::string>>

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
    static void load_instr_targets(TARGETS_TYPE &bb_targets, TARGETS_TYPE &func_targets, TARGETS_TYPE &block_targets, CONST_TARGETS_TYPE &const_targets);

    // -1: not checking, 0: not targets, 1: target BBs, 2: target functions, 3: target blocks, 4: target consts
    static u8 is_target_loc(std::string codefile, unsigned line, TARGETS_TYPE &bb_targets, TARGETS_TYPE &func_targets, TARGETS_TYPE &block_targets, CONST_TARGETS_TYPE &const_targets);

    u8 check_code_language(std::string codefile);
    void printFuncLog(std::string filename, unsigned line, u16 evtID, std::string func_name);
    void printBBLog(std::string filename, unsigned line, u16 evtID);
    void printBlockLog(std::string filename, unsigned line, u16 evtID);
    void printConstLog(std::string filename, unsigned line, u16 evtID, std::string const_name);
    std::vector<std::string> getArgumentTypeDebug(std::vector<std::string> instrumented_parameters, iterator_range<Function::arg_iterator> iterator_arguments);
    std::unordered_map<llvm::Value*, ValueInfo> getArgument(const std::vector<std::string> &instrumented_parameters, iterator_range<Function::arg_iterator> iterator_arguments, std::vector<std::vector<unsigned int>> &default_indices);
    std::vector<llvm::Value*> getValues(std::vector<std::string> &vec, iterator_range<Function::arg_iterator> args, std::vector<std::vector<unsigned int>> &vec_selected_fields, IRBuilder<> &IRB);
    void changeStructPointersToStructTypes(std::unordered_map<Value*, ValueInfo> &valueTypeMap);
  };

}


std::vector<llvm::Value*> AFLCoverage::getValues(
                                    std::vector<std::string> &vec, 
                                    iterator_range<Function::arg_iterator> args, 
                                    std::vector<std::vector<unsigned int>> &vec_selected_fields, IRBuilder<> &IRB
                                  ){
    std::unordered_map<llvm::Value*, ValueInfo> argument_map = getArgument(vec, args, vec_selected_fields);
    std::vector<Value*> res;
    changeStructPointersToStructTypes(argument_map);
    for (auto &pair : argument_map) {
      std::vector<llvm::Value*> tmp;
      llvm::Value* target_value = pair.first;
      llvm::Type* target_type = pair.second.type;
      for(auto &selected_field: pair.second.indexes){
        llvm::Value* zero  = llvm::ConstantInt::get(IRB.getInt32Ty(), 0);
        llvm::Value* offset = llvm::ConstantInt::get(IRB.getInt32Ty(), selected_field);  
        llvm::Value* target_ptr = IRB.CreateGEP(
              target_type,
              target_value,
              {zero, offset}
          );
        target_value = IRB.CreateLoad(target_ptr);
        res.push_back(target_value);
        target_type = target_value->getType();
      }
    }
    return res;
}

/***
 * Load identified interesting basicblocks(targets) to instrument
 ***/
void AFLCoverage::load_instr_targets(TARGETS_TYPE &bb_targets, TARGETS_TYPE &func_targets, TARGETS_TYPE &block_targets, CONST_TARGETS_TYPE &const_targets)
{
  char *target_file = getenv("TARGETS_FILE");
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

    auto targets_block = target["targets_block"];
    for (const auto& block : targets_block) {
      if (block.is_number()) {
        block_targets[codefile].insert(block.get<int>());
      }
    }

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
u8 AFLCoverage::is_target_loc(std::string codefile, unsigned line, TARGETS_TYPE &bb_targets, TARGETS_TYPE &func_targets, TARGETS_TYPE &block_targets, CONST_TARGETS_TYPE &const_targets)
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

  if (func_targets.count(codefile))
  {
    std::set<int> locs = func_targets[codefile];
    for (auto ep = locs.begin(); ep != locs.end(); ep++)
    {
      if (*ep == line)
      {
        func_targets[codefile].erase(line);
        return 2;
      }
    }
  }

  if (block_targets.count(codefile))
  {
    std::set<int> locs = block_targets[codefile];
    for (auto ep = locs.begin(); ep != locs.end(); ep++)
    {
      if (*ep == line)
      {
        block_targets[codefile].erase(line);
        return 3;
      }
    }
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

void AFLCoverage::changeStructPointersToStructTypes(std::unordered_map<Value*, ValueInfo> &valueTypeMap) {
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

std::unordered_map<Value*, ValueInfo> AFLCoverage::getArgument(
    const std::vector<std::string> &instrumented_parameters,
    iterator_range<Function::arg_iterator> iterator_arguments,
    std::vector<std::vector<unsigned int>> &default_indices) 
{
    std::unordered_map<Value*, ValueInfo> valueMap;
    int idx = 0;
    for (auto &Arg : iterator_arguments) {
        for (const std::string &param : instrumented_parameters) {
          
            if (param == Arg.getName().str()) {
                struct ValueInfo valueInfoTmp;
                valueInfoTmp.type = Arg.getType();
                valueInfoTmp.indexes = default_indices[idx];
                valueMap[&Arg] = valueInfoTmp;
            }
        }
      idx++;
    }

    return valueMap;
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

  bbToID.open("/opt/instrumentor/BB2ID.txt", std::ofstream::out | std::ofstream::app);
  if (!bbToID.is_open())
  {
    bbToID.open("./BB2ID.txt", std::ofstream::out | std::ofstream::app);
  }

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

  int inst_blocks = 0;
  TARGETS_TYPE bb_targets, func_targets, block_targets;
  CONST_TARGETS_TYPE const_targets;
  std::set<std::pair<std::string, int>> instrumented_const_targets;
  load_instr_targets(bb_targets, func_targets, block_targets, const_targets);
  u8 codeLang = 0;

  static const std::string Xlibs("/usr/");
  std::ofstream file2("mipass.log", std::ios::app);
  if (!file2) {
      llvm::errs() << "No se pudo abrir mipass.log\n";
      llvm::report_fatal_error("Abortando por error de archivo");  // aborta con core dump (más "ruidoso" que exit)
  }
  for (auto &F : M) {
  //   // Label if this function is instrumented
     bool isTargetFunc = false;

     std::string filename;
     unsigned line = 0;
     unsigned const_line = 0;

    std::string s = "r";
    std::vector<unsigned int> selected_fields = {0,1};
    std::vector<std::string> vec = {s};

    std::vector<std::vector<unsigned int>> vec_selected_fields;
    vec_selected_fields.push_back(selected_fields);

    for (auto &BB : F)
    {
      
      BasicBlock::iterator IP = BB.getFirstInsertionPt();

      // in each basic block, check if it is a target
      bool isTargetBlockEvent = false;
      bool isTargetConstEvent = false;

      for (auto &I : BB)
      {

        get_debug_loc(&I, filename, line);

        if (filename.empty() || line == 0 || !filename.compare(0, Xlibs.size(), Xlibs))
        {
          continue;
        }

        u16 isTarget = is_target_loc(filename, line, bb_targets, func_targets, block_targets, const_targets);
  
        if (isTarget == 2)
        {
          isTargetFunc = true;
        }
        else if (isTarget == 3)
        {
          isTargetBlockEvent = true;
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

      /* instrument starting block point */
      IRBuilder<> IRB(&(*IP));
      std::vector<llvm::Value*> res = getValues(vec, F.args(), vec_selected_fields, IRB);

        file2 << "Funcion: " << F.getName().str() << " " << res.size() << "\n";

        int idx = 0;
        for(int i = 0; i < res.size(); i++){
          std::string typeStr;
          llvm::raw_string_ostream rso(typeStr);
          res[i]->getType()->print(rso);
          rso.flush();
          file2 << " Param " << idx++ << " (" << typeStr << " " << "): " << "\n";
        }

      if (isTargetBlockEvent)
      {
        u16 *evtIDPtr = get_ID_ptr();
        u16 evtID = *evtIDPtr;
        Value *evtValue = ConstantInt::get(Int16Ty, evtID);

        auto *helperTy_stack = FunctionType::get(VoidTy, Int16Ty);
        auto helper_stack_start = M.getOrInsertFunction("trigger_block_event", helperTy_stack);

        IRB.CreateCall(helper_stack_start, {evtValue});

        /* store BB ID info */
        printBlockLog(filename, line, evtID);

        /* increase counter */
        *evtIDPtr = ++evtID;
      }

      if (isTargetConstEvent)
      {

        std::pair<std::string, int> const_key = std::make_pair(filename, const_line);
        if (instrumented_const_targets.find(const_key) == instrumented_const_targets.end())
        {
          instrumented_const_targets.insert(const_key);
          
          std::string constName = const_targets[filename][const_line];
          u16 *evtIDPtr = get_ID_ptr();
          u16 evtID = *evtIDPtr;
          Value *evtValue = ConstantInt::get(Int16Ty, evtID);

          // Create a global string constant for the const name
          Value *constNameValue = IRB.CreateGlobalString(StringRef(constName), "const_name");

          auto *helperTy_const = FunctionType::get(VoidTy, {Int16Ty, Int8PtrTy}, false);
          auto helper_const = M.getOrInsertFunction("trigger_const_event", helperTy_const);

          IRB.CreateCall(helper_const, {evtValue, constNameValue});

          /* store const ID info */
          printConstLog(filename, const_line, evtID, constName);

          /* increase counter */
          *evtIDPtr = ++evtID;
        }
      }

      if (getenv("USE_TRADITIONAL_BRANCH")){
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

      inst_blocks++;
    }

    /* Instrument function if it is one target or the size is above threshold */
    // if (isTargetFunc || F.getInstructionCount() > instr_func_size)
    if (isTargetFunc)
    {

      
      /* get inserting point: first inserting point of the entry block */
      BasicBlock *BB = &F.getEntryBlock();
      Instruction *InsertPoint = &(*(BB->getFirstInsertionPt()));
      IRBuilder<> IRB(InsertPoint);

      /* get evt ID */
      u16 *evtIDPtr = get_ID_ptr();
      u16 evtID = *evtIDPtr;
      Value *evtValue = ConstantInt::get(Int16Ty, evtID);

      auto *helperTy_func = FunctionType::get(VoidTy, Int16Ty);
      auto helper_func = M.getOrInsertFunction("track_functions", helperTy_func);
      IRB.CreateCall(helper_func, {evtValue});

      /* store event ID info */
      get_debug_loc(&(*InsertPoint), filename, line);
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
      *evtIDPtr = ++evtID;
      inst_blocks++;
    }
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
