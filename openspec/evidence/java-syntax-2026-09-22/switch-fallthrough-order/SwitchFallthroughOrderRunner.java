public class SwitchFallthroughOrderRunner {
    public static void main(String[] args) {
        for (int value : new int[] { 9, 1, 4, 0, -2, 10, 9 }) {
            int before = SwitchFallthroughOrder.calls();
            int result = SwitchFallthroughOrder.choose(value);
            System.out.println(value + ":" + result + ":calls:" + (SwitchFallthroughOrder.calls() - before));
        }
    }
}
