public class CandidateContinuation {
    static int mixedExits(int n) {
        int r = 0;
        switch (n) {
            case 1:
                r = 10;
                break;
            case 2:
                return 20;
            default:
                r = 30;
        }
        return r;
    }

}
