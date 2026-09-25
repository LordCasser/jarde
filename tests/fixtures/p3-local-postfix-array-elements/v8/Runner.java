public final class Runner {
    public static void main(String[] args) {
        for (int a : new int[] { -2, 0, 1, 7, Integer.MAX_VALUE }) {
            System.out.println(a + ":" + java.util.Arrays.toString(ArrayPostfixElement.make(a)));
        }
    }
}
