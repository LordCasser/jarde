package cf13;
public class SwitchLoopExits {
    public static int walk(int n) {
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
            score += 3;
        }
        return score;
    }
}
