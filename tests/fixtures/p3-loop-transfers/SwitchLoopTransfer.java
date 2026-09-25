final class SwitchLoopTransfer {
    static int run(int n) {
        int i = 0;
        int total = 0;
        outer:
        while (i < n) {
            switch (i) {
                case 0:
                    if (i == 0) {
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
