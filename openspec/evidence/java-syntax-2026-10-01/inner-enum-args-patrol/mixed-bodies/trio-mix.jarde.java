// jarde: presentation of `p/Trio` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/Trio;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Trio {
    ONE(1, "one", (byte) 4) {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("4303c7067aa40801f766f7fff3d2ef9be99c133a1ebde841385726965f61e69a"), root_container: ContainerId("root"), steps: [] }, ordinal: 12, raw_name: ArchiveNameBytes([112, 47, 84, 114, 105, 111, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("d0df560d55485911eaae5b2b73d53c8594f8c3eee4c3b2429579641af87bccfd"), length: 608 }, variant: Base }
        public java.lang.String describe(int arg1) {
            // @method describe(I)Ljava/lang/String;
            // @declaration an instance method of `p.Trio$1`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return "ONE:" + arg1;
        }
    },
    TWO(2, "two", (byte) 5) {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("4303c7067aa40801f766f7fff3d2ef9be99c133a1ebde841385726965f61e69a"), root_container: ContainerId("root"), steps: [] }, ordinal: 13, raw_name: ArchiveNameBytes([112, 47, 84, 114, 105, 111, 36, 50, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("46b04041a636d3db1f159681e6eb4414e51e0f9d30639d822854ee981baa8296"), length: 630 }, variant: Base }
        public java.lang.String describe(int arg1) {
            // @method describe(I)Ljava/lang/String;
            // @declaration an instance method of `p.Trio$2`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return "TWO:" + arg1;
        }
    };

    private final int code;

    private final java.lang.String label;

    private final byte small;

    private Trio(int arg0, java.lang.String arg1, byte arg2) {
        this.code = arg0;
        this.label = arg1;
        this.small = arg2;
    }

    // jarde: no body: the member `describe(I)Ljava/lang/String;` is declared abstract and its declaration carries no Code attribute
    public abstract java.lang.String describe(int arg1);

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `p.Trio`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) p.Trio.ONE.describe(10)).append(" ").append(p.Trio.ONE.ordinal()).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) p.Trio.TWO.describe(20)).append(" ").append(p.Trio.TWO.ordinal()).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append((java.lang.String) p.Trio.ONE.name()).append(" ").append((java.lang.String) p.Trio.TWO.name()).toString());
        return;
    }
}
