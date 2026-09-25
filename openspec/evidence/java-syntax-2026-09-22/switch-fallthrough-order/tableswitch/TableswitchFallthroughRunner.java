public class TableswitchFallthroughRunner {
    public static void main(String[] args) {
        for (int value : new int[] { 4, 1, 2, 3, 0, -2, 5, 4 }) {
            int before = TableswitchFallthrough.calls();
            int result = TableswitchFallthrough.choose(value);
            System.out.println(value + ":" + result + ":calls:" +
                    (TableswitchFallthrough.calls() - before));
        }
    }
}
