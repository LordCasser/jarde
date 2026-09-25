public final class AccessorVerifyRunner {
    public static void main(String[] args) {
        AccessorVerify value = new AccessorVerify();
        for (int input : new int[] { -1, 2, Integer.MIN_VALUE, Integer.MAX_VALUE }) {
            value.write(input);
            System.out.println(input + ":" + value.value());
        }
    }
}
