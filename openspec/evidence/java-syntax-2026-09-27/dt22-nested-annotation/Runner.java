package dt22;

public class Runner {
    public static void main(String[] args) throws Exception {
        System.out.println("nested=" + Holder.A.class.getMethod("value").getDefaultValue());
        System.out.println("top=" + TopDefault.class.getMethod("value").getDefaultValue()
                + ":" + TopDefault.class.getMethod("count").getDefaultValue());
        System.out.println("dollar=" + Dollar$A.class.getMethod("value").getDefaultValue());
    }
}
