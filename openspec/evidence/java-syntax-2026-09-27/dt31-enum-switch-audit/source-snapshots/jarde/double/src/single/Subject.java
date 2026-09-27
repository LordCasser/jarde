// jarde: presentation of `single/Subject` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package single;

public final class Subject extends java.lang.Object {
    public Subject() {
        // @method <init>()V
        // @declaration a constructor of `single.Subject`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int select(single.Mode arg0) {
        // jarde: enum switch projected at BCI 8 from PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("<normalized>"), root_container: ContainerId("root"), steps: [] }, ordinal: 14, raw_name: ArchiveNameBytes([115, 105, 110, 103, 108, 101, 47, 83, 117, 98, 106, 101, 99, 116, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("cdbf5d15804b082b8c325c4bbcb142791b94e84b0887fb92dcfe91044a26bea8"), length: 496 }, variant: Base }.$SwitchMap$single$Mode via PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("<normalized>"), root_container: ContainerId("root"), steps: [] }, ordinal: 13, raw_name: ArchiveNameBytes([115, 105, 110, 103, 108, 101, 47, 77, 111, 100, 101, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("4584fd7cbdf7331f33e3e40b478992c5ef0b915d450f2549800753ca4ecdcf1d"), length: 815 }, variant: Base }; initializer stores [1=ONE@19[field 12, ordinal 15, read 9, handler 0:23], 2=TWO@34[field 27, ordinal 30, read 24, handler 1:38]]
        // @method select(Lsingle/Mode;)I
        // @declaration a static method of `single.Subject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        switch (arg0) {
            case ONE:
                return 1;
            case TWO:
                return 2;
            default:
                return 0;
        }
    }
}
