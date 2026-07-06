#include <string>
#include <vector>
#include <unordered_map>
#include <memory>
#include "./targets_types.h"

TargetsTypes::TargetsTypes() : impl(std::make_unique<Impl>()) {}

void TargetsTypes::addStructIndexGroups(const FileName& file, LineNum line, const VarName& var, StructIndexGroup group) {
    impl->files[file].lines[line].vars[var].structIndexGroups.push_back(std::move(group));
}

void TargetsTypes::addGroups(const FileName& file, LineNum line, bool endMark, GroupID id) {
    impl->files[file].lines[line].groups = {endMark, id};
}

bool TargetsTypes::containsLine(const FileName& file, LineNum line) const {
    auto f = impl->files.find(file);
    if (f == impl->files.end()) return false;

    auto l = f->second.lines.find(line);
    return l != f->second.lines.end() && !l->second.vars.empty();
}

bool TargetsTypes::getLineEntries(const FileName& file,
                                LineNum line,
                                LineEntries& out) const {

    out.clear();

    auto fIt = impl->files.find(file);
    if (fIt == impl->files.end()) return false;

    auto lIt = fIt->second.lines.find(line);
    if (lIt == fIt->second.lines.end()) return false;

    for (const auto& [var, varNode] : lIt->second.vars) {
        for (const auto& group : varNode.structIndexGroups) {
            out.emplace_back(var, group);
        }
    }

    return true;
}

bool TargetsTypes::isFinalMark(const FileName& file, LineNum line) const {
    auto fIt = impl->files.find(file);
    if (fIt == impl->files.end()) return false;

    auto lIt = fIt->second.lines.find(line);
    if (lIt == fIt->second.lines.end()) return false;

    return lIt->second.groups.endMark;
}

TargetsTypes::GroupID TargetsTypes::getGroupID(const FileName& file, LineNum line) const {
    auto fIt = impl->files.find(file);
    if (fIt == impl->files.end()) return 0;

    auto lIt = fIt->second.lines.find(line);
    if (lIt == fIt->second.lines.end()) return 0;

    return lIt->second.groups.id;
}