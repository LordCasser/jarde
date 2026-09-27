// jarde: presentation of `demo/DoubleOperations` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package demo;

public enum DoubleOperations implements demo.IOps {
    TIMES("*") {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("cc554d6d36e1ad9f9dedee8d1f61a574e31a5e61dd095c18a2da26574873a76e"), root_container: ContainerId("root"), steps: [] }, ordinal: 3, raw_name: ArchiveNameBytes([100, 101, 109, 111, 47, 68, 111, 117, 98, 108, 101, 79, 112, 101, 114, 97, 116, 105, 111, 110, 115, 36, 49, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("baf4794ad32aa9b62fffc574be8a0e2fa692e5d64c8f7391785815b27aa7715e"), length: 642 }, variant: Base }
        public double apply(double x, double y) {
            // @method apply(DD)D
            // @declaration an instance method of `demo.DoubleOperations$1`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return x * y;
        }
    },
    DIVIDE("/") {
        // jarde: selected enum child definition: PhysicalDefinitionId { location: ArchiveEntry { entry: PhysicalEntryId { origin: ContainerOrigin { snapshot: SnapshotId("cc554d6d36e1ad9f9dedee8d1f61a574e31a5e61dd095c18a2da26574873a76e"), root_container: ContainerId("root"), steps: [] }, ordinal: 4, raw_name: ArchiveNameBytes([100, 101, 109, 111, 47, 68, 111, 117, 98, 108, 101, 79, 112, 101, 114, 97, 116, 105, 111, 110, 115, 36, 50, 46, 99, 108, 97, 115, 115]) } }, class_bytes: ClassBytesId { digest: Digest("aad0f0ec1009089e10960357b399c7c7bd90e904c8c1acbb0a69271a10b63adb"), length: 679 }, variant: Base }
        public double apply(double x, double y) {
            // @method apply(DD)D
            // @declaration an instance method of `demo.DoubleOperations$2`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return x / y;
        }
    };

    private final java.lang.String op;

    private DoubleOperations(java.lang.String arg0) {
        this.op = arg0;
    }

    public java.lang.String getOp() {
        // @method getOp()Ljava/lang/String;
        // @declaration an instance method of `demo.DoubleOperations`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.op;
    }
}
