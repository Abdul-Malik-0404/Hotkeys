#include <windows.h>

bool isMenuKeyDown = false;

void simulateKeyPress(WORD vk) {
    INPUT input[2] = {};
    input[0].type = INPUT_KEYBOARD;
    input[0].ki.wVk = vk;
    input[1].type = INPUT_KEYBOARD;
    input[1].ki.wVk = vk;
    input[1].ki.dwFlags = KEYEVENTF_KEYUP;
    SendInput(2, input, sizeof(INPUT));
}

void playPause() {
    simulateKeyPress(VK_MEDIA_PLAY_PAUSE);
}

void nextTrack() {
    simulateKeyPress(VK_MEDIA_NEXT_TRACK);
}

void prevTrack() {
    simulateKeyPress(VK_MEDIA_PREV_TRACK);
}

void volumeUp() {
    simulateKeyPress(VK_VOLUME_UP);
}

void volumeDown() {
    simulateKeyPress(VK_VOLUME_DOWN);
}

void muteUnmute() {
    simulateKeyPress(VK_VOLUME_MUTE);
}

void handleHotkey(int id) {
    if (isMenuKeyDown) {
        switch (id) {
        case 1:
            prevTrack();
            break;
        case 2:
            playPause();
            break;
        case 3:
            nextTrack();
            break;
        case 4:
            volumeUp();
            break;
        case 5:
            volumeDown();
            break;
        case 6:
            muteUnmute();
            break;
        }
    }
}

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    RegisterHotKey(NULL, 1, MOD_NOREPEAT, VK_F1);  // Previous Track
    RegisterHotKey(NULL, 2, MOD_NOREPEAT, VK_F2);  // Play/Pause
    RegisterHotKey(NULL, 3, MOD_NOREPEAT, VK_F3);  // Next Track
    RegisterHotKey(NULL, 4, MOD_NOREPEAT, VK_F7);  // Volume Up
    RegisterHotKey(NULL, 5, MOD_NOREPEAT, VK_F6);  // Volume Down
    RegisterHotKey(NULL, 6, MOD_NOREPEAT, VK_F5);  // Mute/Unmute
    RegisterHotKey(NULL, 7, MOD_NOREPEAT, VK_APPS); // Menu Key

    MSG msg = { 0 };

    while (GetMessage(&msg, NULL, 0, 0)) {
        if (msg.message == WM_HOTKEY) {
            isMenuKeyDown = (GetAsyncKeyState(VK_APPS) & 0x8000) != 0; // Check if Menu key is held down
            if (isMenuKeyDown) {
                handleHotkey(msg.wParam);
            }
        }
    }

    return 0;
}
