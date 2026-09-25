final class SwitchLoopAdjacent {
    static int nested(int start) {
        int i = start;
        int total = 0;
        outer:
        while (i < 3) {
            switch (i) {
                case 0:
                    switch (total) {
                        case 0:
                            total++;
                            break;
                        default:
                            break outer;
                    }
                    break;
                case 1:
                    i++;
                    continue;
                default:
                    break outer;
            }
            i++;
        }
        return total;
    }
}
