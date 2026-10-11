package cf11;

/* JADX INFO: loaded from: UpstreamNestedLoops4.jar:cf11/UpstreamNestedLoops4.class */
public class UpstreamNestedLoops4 {
    public int testFor() {
        int tmp = 1;
        for (int i = 10; i > -1; i--) {
            if (i > tmp) {
                for (int j = 0; j < 54; j += 4) {
                    if (i < j) {
                        for (int k = j; k < j + 4; k++) {
                            if (tmp > k) {
                                return 0;
                            }
                        }
                        break;
                    }
                }
            }
            tmp++;
        }
        return tmp;
    }
}
