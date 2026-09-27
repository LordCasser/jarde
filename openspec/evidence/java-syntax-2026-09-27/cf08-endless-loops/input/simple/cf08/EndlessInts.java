package cf08;

public final class EndlessInts {
    public static int find(int limit) {
        int i = 0;
        while (true) {
            if (i >= limit) {
                break;
            }
            if (i == 3) {
                break;
            }
            i++;
        }
        return i;
    }
}
