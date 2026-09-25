public final class ControlRunner {
    public static void main(String[] args) {
        MixedArrayControls.byteTarget(false, 0);
        MixedArrayControls.plainWrite(false, 0);
        try { MixedArrayControls.nullTarget(false, 0); } catch (NullPointerException expected) { }
        MixedArrayControls.sharedArray(false, 0);
        MixedArrayControls.sharedIndex(false, 0);
        MixedArrayControls.duplicatedValue(false, 0);
        MixedArrayControls.protectedWrite(false, 0);
        MixedArrayControls.guardedWrite(true, false, 0);
        System.out.println("verified");
    }
}
