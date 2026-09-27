package cf13;

/* JADX INFO: loaded from: SwitchLoopExits.class */
public class SwitchLoopExits {
    public static int walk(int n) {
        int score;
        int score2 = 0;
        for (int i = 0; i < n; i++) {
            switch (i % 3) {
                case 0:
                    score = score2 + 1;
                    break;
                case 1:
                    score2 += 10;
                    continue;
                    break;
                default:
                    score = score2 + 2;
                    break;
            }
            score2 = score + 3;
        }
        return score2;
    }
}
