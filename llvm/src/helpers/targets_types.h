#include <string>
#include <vector>
#include <unordered_map>
#include <memory>

class TargetsTypes {
public:

    using StructIndex = unsigned int;
    using StructIndexGroup = std::vector<StructIndex>;
    using StructIndexGroups = std::vector<StructIndexGroup>;
    using GroupID = unsigned int;

    using VarName  = std::string;
    using LineNum  = int;
    using FileName = std::string;

    using VarEntry   = std::pair<VarName, StructIndexGroup>;
    using LineEntries = std::vector<VarEntry>;

    TargetsTypes();

    void addStructIndexGroups(const FileName& file,
             LineNum line,
             const VarName& var,
             StructIndexGroup group);
    
    void addGroups(const FileName& file, LineNum line, bool endMark, GroupID id);

    bool containsLine(const FileName& file, LineNum line) const;

    bool isFinalMark(const FileName& file,
                     LineNum line) const;
    
    GroupID getGroupID(const FileName& file,
                       LineNum line) const;

    bool getLineEntries(const FileName& file,
                        LineNum line,
                        LineEntries& out) const;

private:
    struct Impl;
    std::unique_ptr<Impl> impl;
};

struct TargetsTypes::Impl {

    struct Groups {
        bool endMark;
        GroupID id;
    };

    struct 

    struct VarNode {
        StructIndexGroups structIndexGroups;
    };

    struct LineNode {
        std::unordered_map<VarName, VarNode> vars;
        Groups groups;
    };

    struct FileNode {
        std::unordered_map<LineNum, LineNode> lines;
    };

    std::unordered_map<FileName, FileNode> files;
};
