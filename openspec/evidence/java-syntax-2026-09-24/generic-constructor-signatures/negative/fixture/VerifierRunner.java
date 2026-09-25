package genericctornegative;

public final class VerifierRunner {
    public static void main(String[] args) throws Exception {
        for (String name : args) {
            Class<?> type = Class.forName("genericctornegative." + name);
            if (name.equals("BodyUsesThis")) {
                type.getDeclaredConstructor().newInstance();
            }
            type.getDeclaredConstructor(Number.class).newInstance(Integer.valueOf(7));
            if (name.equals("SameClassCall")) {
                type.getDeclaredMethod("create", Integer.class).invoke(null, Integer.valueOf(8));
            }
            System.out.println("verified=" + name);
        }
    }
}
