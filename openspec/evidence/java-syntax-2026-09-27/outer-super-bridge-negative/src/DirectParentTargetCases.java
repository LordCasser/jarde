public class DirectParentTargetCases extends TargetBaseB {
    @Override int value() { return 202; }

    class Member extends TargetMemberBase {
        int directParentCall() { return DirectParentTargetCases.super.value(); }
    }

    public static void main(String[] args) {
        System.out.println(new DirectParentTargetCases().new Member().directParentCall());
    }
}
class TargetBaseA { int value() { return 101; } }
class TargetBaseB { int value() { return 102; } }
class TargetMemberBase { int value() { return 103; } }
