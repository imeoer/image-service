package attacher

import (
	"context"
	"os"

	"github.com/containerd/containerd/images"
	"github.com/containerd/containerd/reference/docker"
	"github.com/containerd/containerd/remotes"
	"github.com/dragonflyoss/image-service/contrib/nydusify/pkg/converter/provider"

	"github.com/goharbor/acceleration-service/pkg/platformutil"
	"github.com/goharbor/acceleration-service/pkg/remote"
	ocispec "github.com/opencontainers/image-spec/specs-go/v1"
	"github.com/pkg/errors"
)

type Opt struct {
	WorkDir string

	Source string
	Target string

	SourceInsecure bool
	TargetInsecure bool

	AllPlatforms bool
	Platforms    string
}

func hosts(opt Opt) remote.HostFunc {
	maps := map[string]bool{
		opt.Source: opt.SourceInsecure,
		opt.Target: opt.TargetInsecure,
	}
	return func(ref string) (remote.CredentialFunc, bool, error) {
		return remote.NewDockerConfigCredFunc(), maps[ref], nil
	}
}

func fetchManifests(ctx context.Context, pvd *provider.Provider, ref string) error {
	resolver, err := pvd.Resolver(ref)
	if err != nil {
		return errors.Wrap(err, "get resolver")
	}
	_, desc, err := resolver.Resolve(ctx, ref)
	if err != nil {
		return errors.Wrap(err, "resolve image")
	}
	fetcher, err := resolver.Fetcher(ctx, ref)
	if err != nil {
		return errors.Wrap(err, "get fetcher")
	}

	manifestFilterHandler := func(f images.HandlerFunc) images.HandlerFunc {
		return func(ctx context.Context, desc ocispec.Descriptor) ([]ocispec.Descriptor, error) {
			children, err := f(ctx, desc)
			if err != nil {
				return children, err
			}

			var descs []ocispec.Descriptor

			for _, desc := range children {
				if images.IsManifestType(desc.MediaType) || images.IsIndexType(desc.MediaType) {
					descs = append(descs, desc)
				}
			}

			return descs, nil
		}
	}

	handlers := []images.Handler{
		remotes.FetchHandler(pvd.ContentStore(), fetcher),
		manifestFilterHandler(images.ChildrenHandler(pvd.ContentStore())),
	}
	handler := images.Handlers(handlers...)

	if err := images.Dispatch(ctx, handler, nil, desc); err != nil {
		return errors.Wrap(err, "dispatch handler")
	}

	return nil
}

func Attach(ctx context.Context, opt Opt) error {
	platformMC, err := platformutil.ParsePlatforms(opt.AllPlatforms, opt.Platforms)
	if err != nil {
		return err
	}

	if _, err := os.Stat(opt.WorkDir); err != nil {
		if errors.Is(err, os.ErrNotExist) {
			if err := os.MkdirAll(opt.WorkDir, 0755); err != nil {
				return errors.Wrap(err, "prepare work directory")
			}
			// We should only clean up when the work directory not exists
			// before, otherwise it may delete user data by mistake.
			defer os.RemoveAll(opt.WorkDir)
		} else {
			return errors.Wrap(err, "stat work directory")
		}
	}
	tmpDir, err := os.MkdirTemp(opt.WorkDir, "nydusify-")
	if err != nil {
		return errors.Wrap(err, "create temp directory")
	}
	defer os.RemoveAll(tmpDir)

	pvd, err := provider.New(tmpDir, hosts(opt), platformMC)
	if err != nil {
		return err
	}

	sourceNamed, err := docker.ParseDockerRef(opt.Source)
	if err != nil {
		return errors.Wrap(err, "parse source reference")
	}
	targetNamed, err := docker.ParseDockerRef(opt.Target)
	if err != nil {
		return errors.Wrap(err, "parse target reference")
	}
	source := sourceNamed.String()
	target := targetNamed.String()

	if err := fetchManifests(ctx, pvd, source); err != nil {
		return errors.Wrap(err, "fetch source manifests")
	}
	if err := fetchManifests(ctx, pvd, target); err != nil {
		return errors.Wrap(err, "fetch target manifests")
	}

	return nil
}
