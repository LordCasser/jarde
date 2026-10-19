package cf09.literalboolean;

/** Independent refusal cases for literal-only local boolean recovery. */
public final class BooleanLocalRefusals {
    public boolean writeTwo(boolean choose) {
        boolean found = false;
        if (choose) {
            found = true;
        }
        System.out.println("writeTwo: " + found);
        return found;
    }

    public int mixedIntegerSinks(boolean choose) {
        int found = 0;
        if (choose) {
            found = 1;
        }
        System.out.println("mixed: " + found);
        return found + 3;
    }

    public int branchOnly(int choose) {
        int found = 0;
        if (choose != 0) {
            found = 1;
        }
        return found;
    }

    public int parameterOnlyBranch(int value) {
        if (value != 0) {
            return 1;
        }
        return 0;
    }
}
