final class SwitchLoopExits {
    static int run(int start, int limit, boolean stop) {
        int i = start;
        int total = 0;
        outer:
        while (i < limit) {
            switch (i) {
                case 0:
                    if (stop) {
                        break outer;
                    }
                    total++;
                    break;
                case 1:
                    total += 2;
                    break;
                default:
                    break outer;
            }
            i++;
        }
        return total;
    }
}
