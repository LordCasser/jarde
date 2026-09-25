final class PlatformIterableHandlerBoundaryRunner {
    public static void main(String[] args) {
        System.out.println(PlatformIterableHandlerBoundary.caught(
                new PlatformIterableHandlerBoundary.OneShotFailure()));
    }
}
