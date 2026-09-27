public class SharedFinallyOtherTargetRunner {
    public static void main(String[] args) {
        System.out.println(SharedFinallyOtherTarget.handled(false) + ":" + SharedFinallyOtherTarget.count());
        System.out.println(SharedFinallyOtherTarget.handled(true) + ":" + SharedFinallyOtherTarget.count());
    }
}
