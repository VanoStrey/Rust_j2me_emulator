package fixtures;

import javax.microedition.midlet.MIDlet;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Graphics;

public class Fixture7FullscreenAndOrdinaryCanvas extends MIDlet {
    public static int reportedNormalH = 0;
    public static int reportedFullH = 0;

    public static class TestCanvas extends Canvas {
        public TestCanvas() {}
        protected void paint(Graphics g) {
            g.setColor(0x00FFFF00); // Yellow
            g.fillRect(0, 0, getWidth(), getHeight());
        }
    }

    protected void startApp() {
        TestCanvas c = new TestCanvas();
        Display.getDisplay(this).setCurrent(c);
        c.setFullScreenMode(false);
        reportedNormalH = c.getHeight();
        c.setFullScreenMode(true);
        reportedFullH = c.getHeight();
        c.repaint();
    }
    protected void pauseApp() {}
    protected void destroyApp(boolean u) {}
}
