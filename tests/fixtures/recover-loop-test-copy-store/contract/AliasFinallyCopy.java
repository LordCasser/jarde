public class AliasFinallyCopy {
    public static int run(String path) throws Exception {
        ContractResource resource = new ContractResource(path);
        Object alias = AliasState.first;
        int total = 0;
        try {
            int current;
            while ((current = resource.read()) != -1) {
                alias = AliasState.replace(alias, current);
                total = total + current;
            }
        } finally {
            AliasState.observed = alias;
            resource.close();
        }
        return total;
    }
}
