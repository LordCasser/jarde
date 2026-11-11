public final class ChainExtraRunner {
    public static void main(String[] args) {
        for (int mask = 0; mask < 32; mask++) {
            boolean gate = (mask & 1) != 0;
            boolean extra = (mask & 2) != 0;
            boolean left = (mask & 4) != 0;
            boolean other = (mask & 8) != 0;
            ChainExtraBoundary.rhsValue = (mask & 16) != 0;
            ChainExtraBoundary.calls = 0;
            ChainExtraBoundary.assign(gate, extra, left, other);
            System.out.println(mask + ":" + ChainExtraBoundary.result + ":" + ChainExtraBoundary.calls);
        }
    }
}
