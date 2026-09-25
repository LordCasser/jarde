public class ReturnByteRunner {
    public static void main(String[] args) {
        int[] values = new int[] {-2147483648, 2147483647, -65537, -32769, -129, -2, -1, 0, 1, 2, 128, 32768, 65535};
        for (int value : values) {
            System.out.println(value + ":" + ReturnByte.run(value));
        }
    }
}
