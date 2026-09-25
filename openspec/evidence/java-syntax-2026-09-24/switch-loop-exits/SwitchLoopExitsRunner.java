final class SwitchLoopExitsRunner {
    public static void main(String[] args) {
        int[][] cases = {
            {0, 3, 0}, {0, 3, 1}, {1, 3, 0}, {2, 3, 0},
            {0, 1, 0}, {-1, 3, 0}, {3, 3, 0}
        };
        for (int[] row : cases) {
            boolean stop = row[2] != 0;
            System.out.println(row[0] + ":" + row[1] + ":" + stop + ":"
                    + SwitchLoopExits.run(row[0], row[1], stop));
        }
    }
}
