// jarde: presentation of `demo/Op` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

// jarde: class Signature `Ljava/lang/Enum<Ldemo/Op;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Op {
    ADD {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("1fa459d2a44383a1cbf8e011c8e363507e79d5bae1162504096d5e907e1e2dfe"), root_container: ContainerId("root"), steps: [] }, ordinal: 5, raw_name: ArchiveNameBytes([100, 101, 109, 111, 47, 79, 112, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("94b024acddb3c1035914872c6cec42b70a23dec43e8e3a1c2c5b55739f232018"), length: 312 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `demo.Op$1`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 + arg2;
        }
    },
    MULTIPLY {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("1fa459d2a44383a1cbf8e011c8e363507e79d5bae1162504096d5e907e1e2dfe"), root_container: ContainerId("root"), steps: [] }, ordinal: 6, raw_name: ArchiveNameBytes([100, 101, 109, 111, 47, 79, 112, 36, 50, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("ee464b48e528836a487af2a4dd170f2126eddfdb1ae56341aadd25f0f02c7621"), length: 335 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `demo.Op$2`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 * arg2;
        }
    };

    // jarde: no body: the member `apply(II)I` is declared abstract and its declaration carries no Code attribute
    public abstract int apply(int arg1, int arg2);

    public java.lang.String tag() {
        // @method tag()Ljava/lang/String;
        // @declaration an instance method of `demo.Op`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.name() + ":" + this.ordinal();
    }
}
