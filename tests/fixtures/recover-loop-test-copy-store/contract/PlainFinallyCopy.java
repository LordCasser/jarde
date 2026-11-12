public class PlainFinallyCopy {
    public static int run(String path) throws Exception {
        ContractResource resource = new ContractResource(path);
        int total = 0;
        try {
            if ((total = resource.read()) > 0) { total = total + 1; }
            int current;
            while ((current = resource.read()) != -1) { total = total + current; }
        } finally { resource.close(); }
        return total;
    }
}
