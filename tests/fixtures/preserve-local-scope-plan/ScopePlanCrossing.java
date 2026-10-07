public final class ScopePlanCrossing {
    private ScopePlanCrossing() {}

    static int flatFinally(int n) {
        try {
            return 100 / n;
        } catch (Exception caught) {
            return 1;
        } finally {
            System.out.println("done");
        }
    }

    static int resourceAcrossFinally(String path) throws java.io.IOException {
        java.io.BufferedReader reader = new java.io.BufferedReader(
                new java.io.InputStreamReader(new java.io.FileInputStream(path), "UTF-8"));
        try {
            int lines = 0;
            String line;
            while ((line = reader.readLine()) != null) {
                lines++;
            }
            return lines;
        } finally {
            reader.close();
        }
    }
}
