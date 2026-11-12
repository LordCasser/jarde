public class BoundarySkippedNormalCleanup {
    public static int run(String path) throws Exception {
        ContractResource resource = new ContractResource(path);
        int total = 0;
        try {
            int current;
            while ((current = resource.read()) != -1) { total = total + current; }
        } catch (Throwable failure) { resource.close(); throw failure; }
        return total;
    }
}
