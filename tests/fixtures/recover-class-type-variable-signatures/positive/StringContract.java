package classvars;

public final class StringContract implements ClassVariableContract<String> {
    @Override
    public String identity(String value) {
        return value;
    }
}
