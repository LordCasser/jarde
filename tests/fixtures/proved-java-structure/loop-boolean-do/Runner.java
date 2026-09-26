public final class Runner {
    public static void main(String[] args) {
        int[][] inputs = {{3, 3}, {3, -1}, {0, 0}};
        for (int[] input : inputs) {
            System.out.println(input[0] + "," + input[1] + ":"
                    + DoLoopBool.andDo(input[0], input[1]) + ","
                    + DoLoopBool.orDo(input[0], input[1]));
        }
    }
}
