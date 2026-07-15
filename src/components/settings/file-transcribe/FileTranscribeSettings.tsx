import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, FileAudio, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { commands } from "@/bindings";
import { SettingsGroup } from "../../ui/SettingsGroup";
import { Button } from "../../ui/Button";

// Extensions we let the user pick in the file dialog. The backend will still
// accept anything (falling back to ffmpeg for formats our built-in decoder
// can't handle), this list just keeps the picker focused on media files.
const MEDIA_EXTENSIONS = [
  "wav",
  "mp3",
  "m4a",
  "aac",
  "flac",
  "ogg",
  "mp4",
  "mov",
  "mkv",
  "avi",
  "webm",
];

export const FileTranscribeSettings: React.FC = () => {
  const { t } = useTranslation();
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [result, setResult] = useState<string>("");

  const handlePickFile = async () => {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: t("settings.fileTranscribe.filterName"),
          extensions: MEDIA_EXTENSIONS,
        },
      ],
    });
    if (typeof path === "string") {
      setSelectedPath(path);
      setResult("");
    }
  };

  const handleTranscribe = async () => {
    if (!selectedPath) return;
    setIsTranscribing(true);
    setResult("");
    try {
      const res = await commands.transcribeMediaFile(selectedPath);
      if (res.status === "ok") {
        setResult(res.data);
        if (!res.data.trim()) {
          toast.info(t("settings.fileTranscribe.emptyResult"));
        }
      } else {
        toast.error(res.error);
      }
    } catch (e) {
      toast.error(String(e));
    } finally {
      setIsTranscribing(false);
    }
  };

  const handleCopy = async () => {
    if (!result) return;
    await writeText(result);
    toast.success(t("settings.fileTranscribe.copied"));
  };

  const fileName = selectedPath?.split(/[\\/]/).pop() ?? null;

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup
        title={t("settings.fileTranscribe.title")}
        description={t("settings.fileTranscribe.description")}
      >
        <div className="p-4 space-y-4">
          <div className="flex items-center gap-3 flex-wrap">
            <Button
              onClick={handlePickFile}
              variant="secondary"
              size="sm"
              className="flex items-center gap-2"
            >
              <FileAudio className="w-4 h-4" />
              <span>{t("settings.fileTranscribe.chooseFile")}</span>
            </Button>
            {fileName && (
              <span className="text-sm text-text/70 truncate max-w-xs">
                {fileName}
              </span>
            )}
          </div>

          <Button
            onClick={handleTranscribe}
            disabled={!selectedPath || isTranscribing}
            variant="primary"
            size="sm"
            className="flex items-center gap-2"
          >
            {isTranscribing && <Loader2 className="w-4 h-4 animate-spin" />}
            <span>
              {isTranscribing
                ? t("settings.fileTranscribe.transcribing")
                : t("settings.fileTranscribe.transcribe")}
            </span>
          </Button>

          {result && (
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <h3 className="text-xs font-medium text-mid-gray uppercase tracking-wide">
                  {t("settings.fileTranscribe.resultTitle")}
                </h3>
                <button
                  onClick={handleCopy}
                  className="p-1.5 rounded-md flex items-center justify-center transition-colors cursor-pointer text-text/50 hover:text-logo-primary"
                  title={t("settings.fileTranscribe.copy")}
                >
                  <Copy className="w-4 h-4" />
                </button>
              </div>
              <textarea
                readOnly
                value={result}
                rows={6}
                className="w-full text-sm bg-mid-gray/5 border border-mid-gray/20 rounded-lg p-3 resize-y"
              />
            </div>
          )}
        </div>
      </SettingsGroup>
    </div>
  );
};
