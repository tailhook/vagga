use mopa::Any;

use builder::context::Context;
use builder::commands::alpine;
use builder::error::StepError;
use builder::packages;


/// This returns the same as Distribution::name but is separate trait because
/// static methods makes trait non-object-safe
pub trait Named {
    /// Human-readable name of distribution
    fn static_name() -> &'static str;
}

pub trait Distribution: Any {

    /// Only true if distribution is not known yet (i.e. can be set)
    fn is_unknown(&self) -> bool { false }

    /// Human-readable name of distribution
    ///
    /// Object-safe variant of the method
    fn name(&self) -> &'static str;

    /// Downloads initial image of distribution
    fn bootstrap(&mut self, &mut Context) -> Result<(), StepError>;

    /// Does distro-specific cleanup at the end of the build
    fn finish(&mut self, &mut Context) -> Result<(), String> { Ok(()) }

    /// Install normal packages
    fn install(&mut self, &mut Context, &[String]) -> Result<(), StepError>;

    /// Install special predefined packages for specific features
    fn ensure_packages(&mut self, ctx: &mut Context,
        features: &[packages::Package])
        -> Result<Vec<packages::Package>, StepError>;
}

// This is needed for cast to work
mopafy!(Distribution);

pub struct Unknown;

impl Distribution for Unknown {
    fn is_unknown(&self) -> bool { true }
    fn name(&self) -> &'static str { "unknown" }
    fn bootstrap(&mut self, _: &mut Context) -> Result<(), StepError> {
        unreachable!();
    }
    fn install(&mut self, _: &mut Context, _pkgs: &[String])
        -> Result<(), StepError>
    {
        Err(StepError::NoDistro)
    }
    fn ensure_packages(&mut self, _: &mut Context, _: &[packages::Package])
        -> Result<Vec<packages::Package>, StepError>
    {
        Err(StepError::NoDistro)
    }
}

pub trait DistroBox {
    fn set<D: Distribution+Sized>(&mut self, value: D) -> Result<(), StepError>;
    fn specific<T, R, F>(&mut self, f: F) -> Result<R, StepError>
        where T: Distribution+Named, R: Sized,
              F: FnOnce(&mut T) -> Result<R, StepError>;
    fn npm_configure(&mut self, ctx: &mut Context) -> Result<(), StepError>;
}

impl DistroBox for Box<Distribution> {
    fn set<D: Distribution+Sized>(&mut self, value: D) -> Result<(), StepError> {
        if self.is::<Unknown>() {
            *self = Box::new(value);
            Ok(())
        } else {
            return Err(StepError::DistroOverlap(value.name(), self.name()));
        }
    }
    fn specific<T, R, F>(&mut self, f: F) -> Result<R, StepError>
        where T: Distribution+Named, R: Sized,
              F: FnOnce(&mut T) -> Result<R, StepError>,
    {
        self.downcast_mut::<T>()
        .map(f)
        .ok_or(StepError::WrongDistro(T::static_name(), self.name()))
        .and_then(|x| x)
    }
    fn npm_configure(&mut self, ctx: &mut Context) -> Result<(), StepError> {
        if (**self).is::<Unknown>() {
            try!(alpine::configure(self, ctx, alpine::LATEST_VERSION));
        }
        Ok(())
    }
}
