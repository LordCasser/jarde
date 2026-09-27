// jarde: presentation of `grouped/Subject` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package grouped;

public final class Subject extends java.lang.Object {
    public static int trace;

    public Subject() {
        // @method <init>()V
        // @declaration a constructor of `grouped.Subject`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int mark(int arg0) {
        // @method mark(I)I
        // @declaration a static method of `grouped.Subject`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        grouped.Subject.trace = grouped.Subject.trace * 10 + arg0;
        return arg0;
    }

    public static int select(grouped.Count arg0, grouped.Animal arg1) {
        // jarde: enum switch projected at BCI 8 from PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("<normalized>"), root_container: ContainerId("root"), steps: [] }, ordinal: 5, raw_name: ArchiveNameBytes([103, 114, 111, 117, 112, 101, 100, 47, 83, 117, 98, 106, 101, 99, 116, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("b9f8e852bb3fe86c70f0fc93acb7a5f15c72917d01e71b4d361bd85c9b610173"), length: 725 }, variant: Base }.$SwitchMap$grouped$Count via PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("<normalized>"), root_container: ContainerId("root"), steps: [] }, ordinal: 4, raw_name: ArchiveNameBytes([103, 114, 111, 117, 112, 101, 100, 47, 67, 111, 117, 110, 116, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("df3dd61978cb46c2501e17524d2d7f43d9ce8327e3f12c11adbb299e648831fc"), length: 827 }, variant: Base }; initializer stores [1=ONE@58[field 51, ordinal 54, read 48, handler 2:62], 2=TWO@73[field 66, ordinal 69, read 63, handler 3:77]]
        // jarde: enum switch projected at BCI 65 from PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("<normalized>"), root_container: ContainerId("root"), steps: [] }, ordinal: 5, raw_name: ArchiveNameBytes([103, 114, 111, 117, 112, 101, 100, 47, 83, 117, 98, 106, 101, 99, 116, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("b9f8e852bb3fe86c70f0fc93acb7a5f15c72917d01e71b4d361bd85c9b610173"), length: 725 }, variant: Base }.$SwitchMap$grouped$Animal via PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("<normalized>"), root_container: ContainerId("root"), steps: [] }, ordinal: 3, raw_name: ArchiveNameBytes([103, 114, 111, 117, 112, 101, 100, 47, 65, 110, 105, 109, 97, 108, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("145a845f5abf358bef6f10ea3728679a61498f3e22f8e3fd86fdb896f9bc50fd"), length: 785 }, variant: Base }; initializer stores [1=CAT@19[field 12, ordinal 15, read 9, handler 0:23], 2=DOG@34[field 27, ordinal 30, read 24, handler 1:38]]
        // @method select(Lgrouped/Count;Lgrouped/Animal;)I
        // @declaration a static method of `grouped.Subject`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local2;
        switch (arg0) {
            case ONE:
                local2 = mark(1);
                break;
            case TWO:
                local2 = mark(2);
                break;
            default:
                local2 = mark(3);
                break;
        }
        switch (arg1) {
            case CAT:
                local2 = local2 + mark(4);
                break;
            case DOG:
                local2 = local2 + mark(5);
                break;
            default:
                local2 = local2 + mark(6);
                break;
        }
        return local2 * 100 + grouped.Subject.trace;
    }
}
