public class SCGBCompat<T extends java.lang.Comparable<T>> {
    private java.util.Map<java.lang.String, java.util.List<T>> index = new java.util.HashMap<java.lang.String, java.util.List<T>>();
    public static void main(java.lang.String[] args) {
        SCGBCompat<java.lang.String> z = new SCGBCompat<java.lang.String>();
        if (z.index != null) { System.out.println("index:true"); } else { System.out.println("index:false"); }
        java.lang.Object read = z.index;
        if (read instanceof java.util.Map) { System.out.println("read:true"); } else { System.out.println("read:false"); }
    }
}
