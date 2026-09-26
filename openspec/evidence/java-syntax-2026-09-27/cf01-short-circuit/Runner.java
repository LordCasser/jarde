package probe;

public final class Runner {
    public static void main(String[] args) {
        for (int bits = 0; bits < 8; bits++) {
            boolean a = (bits & 4) != 0;
            boolean b = (bits & 2) != 0;
            boolean c = (bits & 1) != 0;
            for (String name : new String[] {"logicalAnd", "logicalOr", "negatedAnd", "negatedOr",
                    "conditions", "negatedConditions", "nested", "negated"}) {
                ShortCircuitNegation.reset();
                boolean value;
                if (name.equals("logicalAnd")) value = ShortCircuitNegation.logicalAnd(a, b);
                else if (name.equals("logicalOr")) value = ShortCircuitNegation.logicalOr(a, b);
                else if (name.equals("negatedAnd")) value = ShortCircuitNegation.negatedAnd(a, b);
                else if (name.equals("negatedOr")) value = ShortCircuitNegation.negatedOr(a, b);
                else if (name.equals("conditions")) value = ShortCircuitNegation.conditions(a, b, c);
                else if (name.equals("negatedConditions")) value = ShortCircuitNegation.negatedConditions(a, b, c);
                else if (name.equals("nested")) value = ShortCircuitNegation.nested(a, b, c);
                else value = ShortCircuitNegation.negated(a, b, c);
                System.out.println(name + ":" + a + "," + b + "," + c + "=" + value + ";" + ShortCircuitNegation.observation());
            }
        }
    }
}
