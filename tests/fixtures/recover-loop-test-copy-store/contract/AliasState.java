public class AliasState {
    public static Object first = new Object(), second = new Object(), observed;
    public static Object replace(Object old, int value) {
        ContractResource.events.append(old == first ? "old-first;" : old == second ? "old-second;" : "old-invalid;");
        if (value == 13) throw ContractResource.readFailure;
        return second;
    }
}
