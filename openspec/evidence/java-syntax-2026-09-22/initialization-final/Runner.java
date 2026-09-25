public class Runner {
    public static void main(String[] args) throws Exception {
        Class<?> type = Class.forName(args[0]);
        System.out.println("constant=" + type.getField("CONSTANT").get(null));
        System.out.println("computed=" + type.getField("COMPUTED").get(null));
        System.out.println("object=" + type.getField("OBJECT").get(null).getClass().getName());
        System.out.println("instance=" + type.getField("instance").get(type.getConstructor(int.class).newInstance(9)));
    }
}
