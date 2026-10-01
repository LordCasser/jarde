// jarde: presentation of `p/Combo` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/Combo;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Combo {
    ADD(1) {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("71aeff158e89a82165f08d811be12d3d811f6b3039f59418ff5638a0e58db72f"), root_container: ContainerId("root"), steps: [] }, ordinal: 3, raw_name: ArchiveNameBytes([112, 47, 67, 111, 109, 98, 111, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("6cda89aab3d71897f35c420c9424b0ebb0361e28f970072c9f0956b584005da6"), length: 319 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `p.Combo$1`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 + arg2;
        }
    },
    MUL(2) {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("71aeff158e89a82165f08d811be12d3d811f6b3039f59418ff5638a0e58db72f"), root_container: ContainerId("root"), steps: [] }, ordinal: 4, raw_name: ArchiveNameBytes([112, 47, 67, 111, 109, 98, 111, 36, 50, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("56a68d60af492197d8b1113514cb789e7b17b8170b279bc5bd48c9ce38010317"), length: 342 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `p.Combo$2`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 * arg2;
        }
    },
    ID(0);

    private final int code;

    private Combo(int arg0) {
        this.code = arg0;
    }

    public int apply(int arg1, int arg2) {
        // @method apply(II)I
        // @declaration an instance method of `p.Combo`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.code;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `p.Combo`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(p.Combo.ADD.apply(3, 4));
        java.lang.System.out.println(p.Combo.MUL.apply(3, 4));
        java.lang.System.out.println(p.Combo.ID.apply(3, 4));
        return;
    }
}
