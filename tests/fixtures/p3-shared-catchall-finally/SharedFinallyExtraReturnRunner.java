public class SharedFinallyExtraReturnRunner {
    public static void main(String[] args) {
        System.out.println(SharedFinallyExtraReturn.handled(false, true) + ":" + SharedFinallyExtraReturn.count());
        System.out.println(SharedFinallyExtraReturn.handled(false, false) + ":" + SharedFinallyExtraReturn.count());
        System.out.println(SharedFinallyExtraReturn.handled(true, false) + ":" + SharedFinallyExtraReturn.count());
    }
}
