/*
 * Decompiled with CFR 0.152.
 * 
 * Could not load the following classes:
 *  javax.microedition.midlet.MIDlet
 */
import javax.microedition.midlet.MIDlet;

public class RMIDlet
extends MIDlet {
    protected void startApp() {
        if (p.a == null) {
            p.a = this;
            p.h(1);
            return;
        }
        p.h(3);
    }

    protected void pauseApp() {
        p.h(2);
        this.notifyPaused();
    }

    protected void destroyApp(boolean bl) {
        p.a();
    }
}

