public class BoundaryChangedCloseTarget {
    public static int run(String path) throws Exception {
        BoundaryResource resource = new BoundaryResource(path);
        int total = 0;
        try {
            int current;
            while ((current = resource.read()) != -1) { total = total + current; }
        } catch (Throwable failure) { resource.finish(); throw failure; }
        resource.close();
        return total;
    }
}
