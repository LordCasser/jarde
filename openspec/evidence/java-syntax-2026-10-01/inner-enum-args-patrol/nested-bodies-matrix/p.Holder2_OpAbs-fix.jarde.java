// jarde: presentation of `p/Holder2$OpAbs` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/Holder2$OpAbs;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Holder2$OpAbs {
    ADD {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("51714bb527e88486d4e9aaaef69f0d022b350ad7ab95a87c696e89e8134e57dd"), root_container: ContainerId("root"), steps: [] }, ordinal: 10, raw_name: ArchiveNameBytes([112, 47, 72, 111, 108, 100, 101, 114, 50, 36, 79, 112, 65, 98, 115, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("9bb35e60bbbed0087210ca2249f2f688a23b210bd7104e812afaf0456056c3b7"), length: 386 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `p.Holder2$OpAbs$1`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 + arg2;
        }
    },
    MUL {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("51714bb527e88486d4e9aaaef69f0d022b350ad7ab95a87c696e89e8134e57dd"), root_container: ContainerId("root"), steps: [] }, ordinal: 11, raw_name: ArchiveNameBytes([112, 47, 72, 111, 108, 100, 101, 114, 50, 36, 79, 112, 65, 98, 115, 36, 50, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("69dc360fbfe9b673bf0f94bf3e12f4fbe9aa0ef1a6a992ebe935edb550aa0abe"), length: 386 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `p.Holder2$OpAbs$2`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 * arg2;
        }
    };

    // jarde: no body: the member `apply(II)I` is declared abstract and its declaration carries no Code attribute
    public abstract int apply(int arg1, int arg2);
}
