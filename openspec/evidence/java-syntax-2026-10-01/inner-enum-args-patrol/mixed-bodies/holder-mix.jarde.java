// jarde: presentation of `p/Holder$Op` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

// jarde: class Signature `Ljava/lang/Enum<Lp/Holder$Op;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public enum Holder$Op {
    PLUS(1) {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("4303c7067aa40801f766f7fff3d2ef9be99c133a1ebde841385726965f61e69a"), root_container: ContainerId("root"), steps: [] }, ordinal: 7, raw_name: ArchiveNameBytes([112, 47, 72, 111, 108, 100, 101, 114, 36, 79, 112, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("1e15fb2889afbad02be64f4f10fc24e3bd822390a88b781588da0592ce5b452f"), length: 379 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `p.Holder$Op$1`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 + arg2;
        }
    },
    MUL(2) {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("4303c7067aa40801f766f7fff3d2ef9be99c133a1ebde841385726965f61e69a"), root_container: ContainerId("root"), steps: [] }, ordinal: 8, raw_name: ArchiveNameBytes([112, 47, 72, 111, 108, 100, 101, 114, 36, 79, 112, 36, 50, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("7b53cbadf421a3d8633d0634005890d7f669f7436705288d7db7abeb4a56b64f"), length: 379 }, variant: Base }
        public int apply(int arg1, int arg2) {
            // @method apply(II)I
            // @declaration an instance method of `p.Holder$Op$2`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return arg1 * arg2;
        }
    },
    ID(0);

    private final int k;

    private Holder$Op(int arg0) {
        this.k = arg0;
    }

    public int apply(int arg1, int arg2) {
        // @method apply(II)I
        // @declaration an instance method of `p.Holder$Op`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.k;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `p.Holder$Op`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(p.Holder$Op.PLUS.apply(3, 4));
        java.lang.System.out.println(p.Holder$Op.MUL.apply(3, 4));
        java.lang.System.out.println(p.Holder$Op.ID.apply(3, 4));
        return;
    }
}
