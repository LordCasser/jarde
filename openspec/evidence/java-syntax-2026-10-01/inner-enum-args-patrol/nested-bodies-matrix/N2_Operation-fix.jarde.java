// jarde: presentation of `N2$Operation` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `Ljava/lang/Enum<LN2$Operation;>;LN2$IOperation;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum N2$Operation implements N2$IOperation {
    PLUS {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("465d2d2c3f52b88fc3a734858ed9cb7de631b5ba5d3904c81a061a0aa87a7df6"), root_container: ContainerId("root"), steps: [] }, ordinal: 6, raw_name: ArchiveNameBytes([78, 50, 36, 79, 112, 101, 114, 97, 116, 105, 111, 110, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("a74951ea5866bff326ed5f49469650bfdaf8b3edd5424977f9d03613ec686a6a"), length: 436 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `N2$Operation$1`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 + arg2;
        }
    },
    MINUS {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("465d2d2c3f52b88fc3a734858ed9cb7de631b5ba5d3904c81a061a0aa87a7df6"), root_container: ContainerId("root"), steps: [] }, ordinal: 7, raw_name: ArchiveNameBytes([78, 50, 36, 79, 112, 101, 114, 97, 116, 105, 111, 110, 36, 50, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("08a8fd6bf85d29bd7c7b13d1c5f5c3962061b10177170d86f449dc38249a052d"), length: 436 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `N2$Operation$2`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 - arg2;
        }
    };
}
