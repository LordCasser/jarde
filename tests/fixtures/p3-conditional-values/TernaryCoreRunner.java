public class TernaryCoreRunner {
    public static void main(String[] args) {
        TernaryCore.reset();
        System.out.println("true=" + TernaryCore.returned(true) + ":trace=" + TernaryCore.trace());
        TernaryCore.reset();
        System.out.println("false=" + TernaryCore.returned(false) + ":trace=" + TernaryCore.trace());
    }
}
