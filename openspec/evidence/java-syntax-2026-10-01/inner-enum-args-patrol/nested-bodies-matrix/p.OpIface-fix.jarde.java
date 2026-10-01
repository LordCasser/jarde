// jarde: presentation of `p/OpIface` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/OpIface;>;Lp/IOperation;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum OpIface implements p.IOperation {
    ADD {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("51714bb527e88486d4e9aaaef69f0d022b350ad7ab95a87c696e89e8134e57dd"), root_container: ContainerId("root"), steps: [] }, ordinal: 16, raw_name: ArchiveNameBytes([112, 47, 79, 112, 73, 102, 97, 99, 101, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("12e111ada4778f90de3c9bf5c7210a77111dd17fe18b17168a3eca5e3c296a95"), length: 318 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `p.OpIface$1`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 + arg2;
        }
    },
    MUL {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("51714bb527e88486d4e9aaaef69f0d022b350ad7ab95a87c696e89e8134e57dd"), root_container: ContainerId("root"), steps: [] }, ordinal: 17, raw_name: ArchiveNameBytes([112, 47, 79, 112, 73, 102, 97, 99, 101, 36, 50, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("d6811b3fee3adfab80f0a775fac041073375f4fee545c61a41fe8053ee306491"), length: 343 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `p.OpIface$2`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 * arg2;
        }
    };
}
