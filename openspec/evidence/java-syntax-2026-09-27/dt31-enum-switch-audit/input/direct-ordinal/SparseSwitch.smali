.class public Ldt31/SparseSwitch;
.super Ljava/lang/Object;

.method public static select(Ldt31/SparseSwitch$Count;)I
    .registers 3
    .param p0, "value"

    invoke-virtual {p0}, Ldt31/SparseSwitch$Count;->ordinal()I
    move-result v0

    sparse-switch v0, :sswitch_data
    const/4 v0, 0x0
    :goto_end
    return v0

    :sswitch_one
    const/4 v0, 0x1
    goto :goto_end

    :sswitch_two
    const/4 v0, 0x2
    goto :goto_end

    :sswitch_data
    .sparse-switch
        0x1 -> :sswitch_one
        0x2 -> :sswitch_two
    .end sparse-switch
.end method
