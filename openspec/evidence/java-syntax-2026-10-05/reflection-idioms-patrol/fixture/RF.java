public class RF {
    static Object load(String cn) throws Exception {                       // Class.forName + newInstance
        return Class.forName(cn).newInstance();
    }
    static Object call(Object target, String m, Object arg) throws Exception {   // getMethod + invoke 反射链
        return target.getClass().getMethod(m, String.class).invoke(target, arg);
    }
    static boolean kind(Object o, String cn) throws Exception {            // isInstance + asSubclass().cast()
        Class<?> c = Class.forName(cn);
        if(c.isInstance(o)){ return c.cast(o).toString().length() > 0; }
        return false;
    }
    static String ctor(String cn, String arg) throws Exception {           // getConstructor + newInstance(带参)
        return (String) Class.forName(cn).getConstructor(String.class).newInstance(arg);
    }
    public static void main(String[] a) throws Exception {
        System.out.println(""+ (load("java.lang.StringBuilder") instanceof StringBuilder)
            + "/" + call(new StringBuilder("ab"), "append", "c")
            + "/" + kind("xy", "java.lang.String")
            + "/" + ctor("java.lang.String", "made"));
    }
}
