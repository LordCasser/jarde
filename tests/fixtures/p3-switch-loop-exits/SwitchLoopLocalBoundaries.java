public class SwitchLoopLocalBoundaries {
    public static int ordinaryTail(int n) {
        int score = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 3) {
                case 0:
                    score += 1;
                    break;
                case 1:
                    score += 2;
                    break;
                default:
                    score += 3;
                    break;
            }
            score += 4;
        }
        return score;
    }

    public static int crossCase(int n) {
        int score = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 4) {
                case 0:
                    score += 1;
                case 1:
                    score += 2;
                    break;
                case 2:
                    score += 10;
                    continue;
                default:
                    score += 3;
                    break;
            }
            score += 4;
        }
        return score;
    }

    public static int extraEntry(int n) {
        int score = 0;
        for (int i = 0; i < n; i++) {
            if (i != 4) {
                switch (i % 3) {
                    case 0:
                        score += 1;
                        break;
                    case 1:
                        score += 10;
                        continue;
                    default:
                        score += 2;
                        break;
                }
            }
            score += 3;
        }
        return score;
    }

    public static int multipleJoins(int n) {
        int score = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 3) {
                case 0:
                    score += 1;
                    break;
                case 1:
                    score += 10;
                    continue;
                default:
                    score += 2;
                    break;
            }
            if (i % 2 == 0) {
                score += 3;
            } else {
                score += 4;
            }
            score += 5;
        }
        return score;
    }
}
