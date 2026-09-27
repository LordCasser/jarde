package cf08;

public final class Runner {
    public static void main(String[] args) {
        for (int limit : new int[] {0, 1, 2, 3, 4, 8}) {
            System.out.println(EndlessInts.find(limit));
        }
    }
}
