import { createImageBlockConfig, imageParse } from "@blocknote/core";
import {
  createReactBlockSpec,
  type ReactCustomBlockRenderProps,
  ResizableFileBlockWrapper,
  useResolveUrl,
} from "@blocknote/react";
import { MediaLoadingPlaceholder } from "./MediaLoadingPlaceholder";
import { useMediaLoader } from "./useMediaLoader";

function ImagePreview(
  props: Omit<ReactCustomBlockRenderProps<typeof createImageBlockConfig>, "contentRef">,
) {
  const url = props.block.props.url;
  const resolved = useResolveUrl(url ?? "");
  const fileName = props.block.props.name || url?.split("/").pop() || "image";

  const { state, src, retry } = useMediaLoader(
    resolved.loadingState === "loaded" ? resolved.downloadUrl : undefined,
    fileName,
  );

  if (!url) return null;

  if (state === "loaded" && src) {
    return (
      <img
        className="bn-visual-media"
        src={src}
        alt={props.block.props.caption || "BlockNote image"}
        contentEditable={false}
        draggable={false}
        style={{ opacity: 1, transition: "opacity 0.3s ease-in" }}
      />
    );
  }

  return <MediaLoadingPlaceholder type="image" fileName={fileName} state={state} onRetry={retry} />;
}

function ImageBlockRender(props: ReactCustomBlockRenderProps<typeof createImageBlockConfig>) {
  return (
    <ResizableFileBlockWrapper {...(props as any)}>
      <ImagePreview {...(props as any)} />
    </ResizableFileBlockWrapper>
  );
}

export const CustomReactImageBlock = createReactBlockSpec(createImageBlockConfig, (config) => ({
  meta: {
    fileBlockAccept: ["image/*"],
  },
  render: ImageBlockRender,
  parse: imageParse(config),
  runsBefore: ["file"],
}));
