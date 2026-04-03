import { createVideoBlockConfig, videoParse } from "@blocknote/core";
import {
  createReactBlockSpec,
  type ReactCustomBlockRenderProps,
  ResizableFileBlockWrapper,
  useResolveUrl,
} from "@blocknote/react";
import { MediaLoadingPlaceholder } from "./MediaLoadingPlaceholder";
import { useMediaLoader } from "./useMediaLoader";

function VideoPreview(
  props: Omit<ReactCustomBlockRenderProps<typeof createVideoBlockConfig>, "contentRef">,
) {
  const url = props.block.props.url;
  const resolved = useResolveUrl(url ?? "");
  const fileName = props.block.props.name || url?.split("/").pop() || "video";

  // For video, we probe with an Image on the poster/first-frame, but actually
  // just check if the file is fetchable. Using the same probe mechanism.
  const { state, src, retry } = useMediaLoader(
    resolved.loadingState === "loaded" ? resolved.downloadUrl : undefined,
    fileName,
  );

  if (!url) return null;

  if (state === "loaded" && src) {
    return (
      <video
        className="bn-visual-media"
        src={src}
        controls={true}
        contentEditable={false}
        draggable={false}
        style={{ opacity: 1, transition: "opacity 0.3s ease-in" }}
      >
        <track kind="captions" />
      </video>
    );
  }

  return <MediaLoadingPlaceholder type="video" fileName={fileName} state={state} onRetry={retry} />;
}

function VideoBlockRender(props: ReactCustomBlockRenderProps<typeof createVideoBlockConfig>) {
  return (
    <ResizableFileBlockWrapper {...(props as any)}>
      <VideoPreview {...(props as any)} />
    </ResizableFileBlockWrapper>
  );
}

export const CustomReactVideoBlock = createReactBlockSpec(createVideoBlockConfig, (config) => ({
  meta: {
    fileBlockAccept: ["video/*"],
  },
  render: VideoBlockRender,
  parse: videoParse(config),
  runsBefore: ["file"],
}));
