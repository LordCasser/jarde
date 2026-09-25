package nobodygeneric;

import java.io.IOException;

public final class NoBodyGenericCaller {
    static final class Identity extends NoBodyGenericIdentity {
        @Override
        public <T extends Number> T echo(T value) {
            return value;
        }

        @Override
        public <T extends Number> T checked(T value) throws IOException {
            return value;
        }
    }

    public static void main(String[] args) throws Exception {
        NoBodyGenericIdentity value = new Identity();
        Integer echo = value.echo(Integer.valueOf(7));
        Integer checked = value.checked(Integer.valueOf(8));
        System.out.println("values=" + echo + "," + checked);
        System.out.println("returns=" + NoBodyGenericIdentity.class
                .getDeclaredMethod("echo", Number.class).getGenericReturnType().getTypeName()
                + "," + NoBodyGenericIdentity.class
                .getDeclaredMethod("checked", Number.class).getGenericReturnType().getTypeName());
        System.out.println("throws=" + NoBodyGenericIdentity.class
                .getDeclaredMethod("checked", Number.class)
                .getGenericExceptionTypes()[0].getTypeName());
    }
}
